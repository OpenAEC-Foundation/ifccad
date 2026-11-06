use ifccad::*;

fn drawing() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}

fn snapshot(commands: Vec<IfccadLeaderGraphicCommand>) -> IfccadLeaderGraphics {
    IfccadLeaderGraphics {
        origin: IfccadLeaderGraphicsOrigin::SourceSnapshot,
        placement: IfccadPlacement {
            origin: [0.; 3],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
        commands,
    }
}

#[test]
fn saved_graphics_keep_command_order_and_balance_transform_stack() {
    use IfccadLeaderGraphicCommand as C;
    let identity = [
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ];
    let d = drawing();
    let valid = snapshot(vec![
        C::SetSelectionMarker(15001),
        C::PushTransform(identity),
        C::CadCompatibility51Zero,
        C::PopTransform,
    ]);
    valid.validate(&d).unwrap();
    assert!(snapshot(vec![C::PopTransform]).validate(&d).is_err());
    assert!(snapshot(vec![C::PushTransform(identity)])
        .validate(&d)
        .is_err());
    let mut invalid = identity;
    invalid[12] = f64::NAN;
    assert!(snapshot(vec![C::PushTransform(invalid), C::PopTransform])
        .validate(&d)
        .is_err());
    assert_eq!(valid.commands.len(), 4);
    assert!(matches!(valid.commands[2], C::CadCompatibility51Zero));
}

#[test]
fn saved_graphics_use_native_resource_ids_and_distinguish_inheritance_modes() {
    use IfccadLeaderGraphicCommand as C;
    use IfccadLeaderGraphicInheritance as I;
    let d = drawing();
    snapshot(vec![
        C::SetLayer(Some(d.layers[0].id)),
        C::SetColor(I::Owner),
        C::SetColor(I::ByLayer),
        C::SetColor(I::ByBlock),
    ])
    .validate(&d)
    .unwrap();
    assert_ne!(I::<IfccadColor>::Owner, I::ByBlock);
    assert!(snapshot(vec![C::SetLayer(Some(u64::MAX))])
        .validate(&d)
        .is_err());
    assert!(
        snapshot(vec![C::SetLinePattern(I::Explicit(IfccadLinePatternId(
            u64::MAX
        )))])
        .validate(&d)
        .is_err()
    );
}

#[test]
fn graphic_text_keeps_empty_font_requests_and_literal_processing() {
    let d = drawing();
    let text = IfccadLeaderGraphicText {
        position: [0.; 3],
        normal: [0., 0., 1.],
        direction: [1., 0., 0.],
        text: "%%c 🙂".into(),
        processing: IfccadLeaderGraphicTextProcessing::Raw,
        height: 2.5,
        width_factor: 1.,
        oblique_angle: 0.,
        tracking: 1.,
        backwards: false,
        upside_down: false,
        vertical: true,
        underline: false,
        overline: false,
        font: Some(IfccadLeaderGraphicFont {
            family: String::new(),
            font_file: String::new(),
            big_font_file: String::new(),
            bold: Some(false),
            italic: Some(false),
            charset: Some(1),
            pitch_family: Some(34),
        }),
    };
    let value = snapshot(vec![IfccadLeaderGraphicCommand::Draw(
        IfccadLeaderGraphicPrimitive::Text(Box::new(text.clone())),
    )]);
    value.validate(&d).unwrap();
    let mut bad = text;
    bad.text.push('\0');
    assert!(snapshot(vec![IfccadLeaderGraphicCommand::Draw(
        IfccadLeaderGraphicPrimitive::Text(Box::new(bad))
    )])
    .validate(&d)
    .is_err());
}

fn dimension_drawing() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/dimension-kinds-gallery.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}

#[test]
fn classic_leader_keeps_detached_annotation_intent_and_stored_context() {
    let mut d = dimension_drawing();
    d.annotation_scales.push(IfccadAnnotationScale {
        id: IfccadAnnotationScaleId(900),
        name: "leader-test".into(),
        paper_units: 1.,
        drawing_units: 20.,
        unit_scale: false,
    });
    let context = IfccadLeaderAnnotationContext {
        points: vec![[30., 40., 0.], [50., 60., 0.]],
        horizontal_direction: [0., 1., 0.],
        annotation_enabled: false,
        insertion_offset: [7., 8., 0.],
        endpoint_projection: [9., 10., 0.],
    };
    let mut leader = IfccadLeader {
        placement: snapshot(vec![]).placement,
        path: IfccadLeaderPath::SplineThroughVertices(vec![[0.; 3], [1., 2., 0.]]),
        annotation_kind: IfccadLeaderAnnotationKind::MText,
        annotation_entity: None,
        dimension_style: d.dimension_styles[0].id,
        overrides: Default::default(),
        arrow: None,
        arrow_enabled: true,
        hook_enabled: false,
        hook_direction: IfccadLeaderHookDirection::OppositeHorizontal,
        annotation_box: Some([0., 2.5]),
        horizontal_direction: [1., 0., 0.],
        block_offset: [1., 2., 3.],
        annotation_offset: [4., 5., 6.],
        origin: [7., 8., 9.],
        annotation: Some(IfccadAnnotation {
            orientation: None,
            mode: IfccadAnnotationMode::Enabled,
            reference: None,
            contexts: vec![IfccadAnnotationContext {
                key: IfccadAnnotationContextKey {
                    scale_id: IfccadAnnotationScaleId(900),
                },
                presentation: Some(context.clone()),
            }],
        }),
        graphics: None,
    };
    leader.validate_values(&d).unwrap();
    assert_eq!(
        leader.annotation.as_ref().unwrap().contexts[0].presentation,
        Some(context)
    );
    assert_eq!(leader.path.vertices(), &[[0.; 3], [1., 2., 0.]]);
    leader.horizontal_direction = [0.; 3];
    assert!(leader.validate_values(&d).is_err());
    leader.horizontal_direction = [1., 0., 0.];
    leader.arrow = Some(IfccadLeaderArrow::Symbol {
        definition_id: u64::MAX,
    });
    assert!(leader.validate_values(&d).is_err());
}

#[test]
fn tolerance_keeps_control_notation_and_validates_saved_scale_pose() {
    let mut d = dimension_drawing();
    d.annotation_scales.push(IfccadAnnotationScale {
        id: IfccadAnnotationScaleId(901),
        name: "fcf-test".into(),
        paper_units: 1.,
        drawing_units: 5.,
        unit_scale: false,
    });
    let code = "{\\Fgdt;p}%%v0.05%%vA%%v%%v^J%%v🙂\r\n";
    let mut tolerance = IfccadTolerance {
        placement: snapshot(vec![]).placement,
        dimension_style: d.dimension_styles[0].id,
        overrides: Default::default(),
        notation: IfccadToleranceNotation::CadFeatureControlTextV1 { code: code.into() },
        annotation: Some(IfccadAnnotation {
            orientation: None,
            mode: IfccadAnnotationMode::Enabled,
            reference: None,
            contexts: vec![IfccadAnnotationContext {
                key: IfccadAnnotationContextKey {
                    scale_id: IfccadAnnotationScaleId(901),
                },
                presentation: Some(IfccadToleranceAnnotationContext {
                    location: [10., 20., 30.],
                    horizontal_direction: [0., 1., 0.],
                }),
            }],
        }),
        graphics: None,
    };
    tolerance.validate_values(&d).unwrap();
    assert_eq!(tolerance.notation.code(), code);
    tolerance.notation = IfccadToleranceNotation::CadFeatureControlTextV1 {
        code: "%<unbound-field>%".into(),
    };
    assert!(tolerance.validate_values(&d).is_err());
    tolerance.notation = IfccadToleranceNotation::CadFeatureControlTextV1 { code: code.into() };
    tolerance.annotation.as_mut().unwrap().contexts[0]
        .presentation
        .as_mut()
        .unwrap()
        .horizontal_direction = [0.; 3];
    assert!(tolerance.validate_values(&d).is_err());
}

#[test]
fn multileader_branch_keys_and_effective_segment_breaks_are_independent_of_source_indices() {
    use std::collections::BTreeSet;
    let d = drawing();
    let line = IfccadMultiLeaderLine {
        key: 9,
        source_index: Some(900),
        settings: IfccadMultiLeaderLineSettings {
            path_kind: IfccadMultiLeaderPathKind::SplineThroughVertices,
            color: IfccadMode::ByBlock,
            line_pattern: IfccadMode::ByLayer,
            line_weight: IfccadLeaderLineWeight::DrawingDefault,
            arrow: IfccadLeaderArrow::DefaultClosedFilled,
            arrow_size: 2.5,
        },
        overridden_properties: BTreeSet::from([IfccadMultiLeaderLineProperty::PathKind]),
        vertices: vec![[1., 2., 0.]],
        connection: IfccadMultiLeaderConnection::GroupConnection,
        breaks: vec![IfccadMultiLeaderSegmentBreaks {
            segment_index: 0,
            intervals: vec![IfccadLeaderBreakInterval {
                start: [3., 4., 0.],
                end: [5., 6., 0.],
            }],
        }],
    };
    let mut groups = vec![IfccadMultiLeaderGroup {
        key: 4,
        source_index: Some(900),
        has_connection_point: true,
        connection_point: [7., 8., 0.],
        has_dogleg_direction: false,
        dogleg_direction: [0.; 3],
        dogleg_length: 0.,
        text_attachment_direction: IfccadMultiLeaderAttachmentDirection::Horizontal,
        lines: vec![line],
        breaks: vec![],
    }];
    validate_ifccad_multi_leader_branches(&d, &groups, 10).unwrap();
    assert_eq!(groups[0].lines[0].vertices.len(), 1);
    assert_eq!(groups[0].lines[0].source_index, groups[0].source_index);
    // Segment 0 includes the separate group endpoint; there is no segment 1.
    groups[0].lines[0].breaks[0].segment_index = 1;
    assert!(validate_ifccad_multi_leader_branches(&d, &groups, 10).is_err());
    groups[0].lines[0].breaks[0].segment_index = 0;
    groups[0].lines[0].key = 4;
    assert!(validate_ifccad_multi_leader_branches(&d, &groups, 10).is_err());
    groups[0].lines[0].key = 9;
    assert!(validate_ifccad_multi_leader_branches(&d, &groups, 9).is_err());
    groups[0].has_connection_point = false;
    assert!(validate_ifccad_multi_leader_branches(&d, &groups, 10).is_err());
    groups[0].has_connection_point = true;
    groups[0].has_dogleg_direction = true;
    assert!(validate_ifccad_multi_leader_branches(&d, &groups, 10).is_err());
}

#[test]
fn multileader_block_labels_bind_only_to_direct_definitions_without_insert_attributes() {
    use std::collections::BTreeSet;
    let mut d = dimension_drawing();
    // Copying an existing definition into the fixture is unnecessary: an ATTDEF
    // binding must be checked by entity identity inside the declared block.
    let definition = IfccadAttributeDefinition {
        tag: "DETAIL".into(),
        prompt: String::new(),
        text: IfccadText {
            annotation: None,
            style_id: d.text_styles[0].id,
            placement: snapshot(vec![]).placement,
            rotation: 0.,
            backward: false,
            upside_down: false,
            layout: ifccad::text::TextLayout::Anchored {
                horizontal: ifccad::text::TextHorizontalAlignment::Left,
                vertical: ifccad::text::TextVerticalAlignment::Baseline,
                height: 2.5,
                width_factor: 1.,
            },
            oblique_angle: 0.,
            thickness: 0.,
            content: vec![],
        },
        flags: Default::default(),
        lock_position: false,
        field_length: 0,
        m_text: None,
        annotation: None,
        secondary_attributes: vec![],
        fields: None,
    };
    let mut entity = d
        .model
        .entities
        .iter()
        .find_map(IfccadEntity::as_native)
        .unwrap()
        .clone();
    entity.id = 800;
    entity.kind = IfccadEntityKind::AttributeDefinition(Box::new(definition));
    let mut block = d
        .blocks
        .first()
        .cloned()
        .unwrap_or_else(|| IfccadBlockDefinition {
            annotation: Default::default(),
            description: String::new(),
            anonymous: false,
            explodable: true,
            uniform_scaling: false,
            bounds_quality: None,
            id: 600,
            name: "labels".into(),
            base_point: [0.; 3],
            insertion_unit: "unitless".into(),
            bounds: None,
            entities: vec![],
        });
    block.id = 600;
    block.uniform_scaling = false;
    block.entities = vec![IfccadEntity::Native(entity)];
    d.blocks.push(block);
    let mut labels = vec![IfccadMultiLeaderBlockLabel {
        key: 3,
        definition_id: Some(800),
        source_index: Some(30),
        width: 0.,
        text: "A-12".into(),
    }];
    validate_ifccad_multi_leader_labels(&d, &labels, 4, &BTreeSet::from([600])).unwrap();
    labels[0].definition_id = None;
    validate_ifccad_multi_leader_labels(&d, &labels, 4, &BTreeSet::from([600])).unwrap();
    labels[0].definition_id = Some(800);
    assert!(validate_ifccad_multi_leader_labels(&d, &labels, 4, &BTreeSet::new()).is_err());
    assert!(validate_ifccad_multi_leader_labels(&d, &labels, 3, &BTreeSet::from([600])).is_err());
    let mut content = IfccadMultiLeaderBlockContent {
        definition_id: 600,
        transform: IfccadBlockTransform {
            placement: snapshot(vec![]).placement,
            rotation: 0.5,
            scale: [-1., 2., 3.],
        },
        color: IfccadMode::ByBlock,
        connection: IfccadMultiLeaderBlockConnection::BasePoint,
        stored_transform: Some([0.; 16]),
    };
    content.validate(&d).unwrap();
    assert_eq!(content.transform.scale, [-1., 2., 3.]);
    content.stored_transform.as_mut().unwrap()[4] = f64::NAN;
    assert!(content.validate(&d).is_err());
}

#[test]
fn multileader_dormant_content_is_independent_and_nested_text_has_no_scale_registry() {
    let d = dimension_drawing();
    let text = IfccadMText {
        annotation: None,
        style_id: d.text_styles[0].id,
        placement: snapshot(vec![]).placement,
        rotation: 0.25,
        backward: false,
        upside_down: false,
        height: 2.5,
        attachment: ifccad::text::MTextAttachment::TopLeft,
        flow: ifccad::text::MTextFlow::Horizontal,
        wrap_width: None,
        columns: None,
        background: None,
        character_format: Default::default(),
        paragraph_format: Default::default(),
        content: vec![Default::default()],
    };
    let attachments = IfccadMultiLeaderTextAttachments {
        left: IfccadMultiLeaderTextAttachment::MiddleOfText,
        right: IfccadMultiLeaderTextAttachment::BottomLine,
        top: IfccadMultiLeaderTextAttachment::CenterOfText,
        bottom: IfccadMultiLeaderTextAttachment::CenterOfTextOverline,
    };
    let mut presentation = IfccadMultiLeaderPresentation {
        plane: snapshot(vec![]).placement,
        normal_reversed: true,
        content_base_point: [10., 20., 30.],
        scale_factor: 2.,
        text_height: 4.,
        arrow_size: 3.,
        landing_gap: 0.,
        next_part_id: 1,
        groups: vec![],
        content: IfccadMultiLeaderStoredContent {
            m_text: Some(Box::new(IfccadMultiLeaderTextContent {
                text,
                state: IfccadMultiLeaderTextState {
                    stored_rotation: Some(-0.375),
                    boundary_height: 0.,
                    automatic_height: false,
                    word_break: true,
                    color: IfccadMode::ByBlock,
                    attachments,
                    attachment_point: IfccadMultiLeaderTextAttachmentPoint::Right,
                    alignment: IfccadMultiLeaderTextAlignment::Center,
                    background: IfccadMultiLeaderTextBackground {
                        enabled: false,
                        mask_enabled: true,
                        color: IfccadMode::Explicit(IfccadColor::Foreground),
                        scale_factor: 1.5,
                        opacity: IfccadMode::Explicit(0.75),
                    },
                },
            })),
            block: None,
            tolerance: Some(Box::new(IfccadMultiLeaderToleranceContent {
                placement: snapshot(vec![]).placement,
                dimension_style: d.dimension_styles[0].id,
                overrides: Default::default(),
                notation: IfccadToleranceNotation::CadFeatureControlTextV1 {
                    code: "%%v0.1%%vA".into(),
                },
            })),
        },
    };
    presentation.validate(&d).unwrap();
    assert_ne!(
        presentation.text_height,
        presentation.content.m_text.as_ref().unwrap().text.height
    );
    assert!(presentation.content.tolerance.is_some());
    presentation
        .content
        .m_text
        .as_mut()
        .unwrap()
        .text
        .annotation = Some(Box::new(IfccadAnnotation {
        orientation: None,
        mode: IfccadAnnotationMode::Disabled,
        reference: None,
        contexts: vec![],
    }));
    assert!(presentation.validate(&d).is_err());
    presentation
        .content
        .m_text
        .as_mut()
        .unwrap()
        .text
        .annotation = None;
    presentation
        .content
        .m_text
        .as_mut()
        .unwrap()
        .state
        .background
        .opacity = IfccadMode::Explicit(1.01);
    assert!(presentation.validate(&d).is_err());
}

#[test]
fn multileader_style_and_entity_settings_do_not_replace_saved_scale_presentations() {
    let mut d = dimension_drawing();
    d.annotation_scales.push(IfccadAnnotationScale {
        id: IfccadAnnotationScaleId(999),
        name: "independent".into(),
        paper_units: 1.,
        drawing_units: 20.,
        unit_scale: false,
    });
    let style = IfccadMultiLeaderStyle::new(IfccadMultiLeaderStyleId(41), "notes");
    let mut leader = IfccadMultiLeader::new();
    leader.style_id = Some(style.id);
    leader.settings.common.line.arrow_size = 9.;
    leader.presentation.arrow_size = 5.;
    leader
        .overridden_properties
        .insert(IfccadMultiLeaderProperty::ArrowSize);
    let mut context = leader.presentation.clone();
    context.arrow_size = 7.;
    context.scale_factor = 20.;
    leader.annotation = Some(IfccadAnnotation {
        orientation: None,
        mode: IfccadAnnotationMode::Disabled,
        reference: None,
        contexts: vec![IfccadAnnotationContext {
            key: IfccadAnnotationContextKey {
                scale_id: IfccadAnnotationScaleId(999),
            },
            presentation: Some(context),
        }],
    });
    validate_ifccad_multi_leader_styles(&d, std::slice::from_ref(&style), 42).unwrap();
    leader
        .validate_values(&d, std::slice::from_ref(&style))
        .unwrap();
    assert_eq!(leader.presentation.arrow_size, 5.);
    assert_eq!(
        leader.annotation.as_ref().unwrap().contexts[0]
            .presentation
            .as_ref()
            .unwrap()
            .arrow_size,
        7.
    );
    assert_eq!(leader.settings.common.line.arrow_size, 9.);
    assert_ne!(style.common.line.arrow_size, 9.);
    assert!(leader.validate_values(&d, &[]).is_err());
    let mut duplicate = style.clone();
    duplicate.id = IfccadMultiLeaderStyleId(40);
    duplicate.name = "NOTES".into();
    assert!(validate_ifccad_multi_leader_styles(&d, &[style, duplicate], 42).is_err());
}

#[test]
fn complete_leader_kinds_use_production_ifcx_and_cbor_readers() {
    let mut d = dimension_drawing();
    d.model.bounds = None;
    d.model.bounds_quality = None;
    let style_id = d.id_counters.allocate_multi_leader_style_id().unwrap();
    d.multi_leader_styles
        .push(IfccadMultiLeaderStyle::new(style_id, "leader styles"));
    let tolerance_id = d.id_counters.allocate_entity_id().unwrap();
    let leader_id = d.id_counters.allocate_entity_id().unwrap();
    let multi_id = d.id_counters.allocate_entity_id().unwrap();
    let make = |id, kind| {
        IfccadEntity::Native(IfccadNativeEntity {
            id,
            visible: true,
            layer_id: d.layers[0].id,
            appearance: IfccadEntityAppearance {
                color: IfccadMode::ByLayer,
                opacity: IfccadMode::ByLayer,
                line_pattern: IfccadMode::ByLayer,
                line_weight: IfccadMode::ByLayer,
            },
            line_pattern_scale: 1.,
            kind,
        })
    };
    let tolerance = IfccadTolerance {
        placement: snapshot(vec![]).placement,
        dimension_style: d.dimension_styles[0].id,
        overrides: Default::default(),
        notation: IfccadToleranceNotation::CadFeatureControlTextV1 {
            code: "{\\Fgdt;p}%%v0.1%%vA".into(),
        },
        annotation: None,
        graphics: None,
    };
    let leader = IfccadLeader {
        placement: snapshot(vec![]).placement,
        path: IfccadLeaderPath::StraightSegments(vec![[0.; 3], [5., 6., 0.]]),
        annotation_kind: IfccadLeaderAnnotationKind::Tolerance,
        annotation_entity: Some(tolerance_id),
        dimension_style: d.dimension_styles[0].id,
        overrides: Default::default(),
        arrow: None,
        arrow_enabled: true,
        hook_enabled: false,
        hook_direction: IfccadLeaderHookDirection::AlongHorizontal,
        annotation_box: None,
        horizontal_direction: [1., 0., 0.],
        block_offset: [0.; 3],
        annotation_offset: [0.; 3],
        origin: [0.; 3],
        annotation: None,
        graphics: Some(snapshot(vec![
            IfccadLeaderGraphicCommand::CadCompatibility51Zero,
        ])),
    };
    let mut multi = IfccadMultiLeader::new();
    multi.style_id = Some(style_id);
    multi.presentation.content.tolerance = Some(Box::new(IfccadMultiLeaderToleranceContent {
        placement: tolerance.placement.clone(),
        dimension_style: tolerance.dimension_style,
        overrides: Default::default(),
        notation: tolerance.notation.clone(),
    }));
    multi.settings.common.content_kind = IfccadMultiLeaderContentKind::Tolerance;
    d.model.entities.extend([
        make(
            tolerance_id,
            IfccadEntityKind::Tolerance(Box::new(tolerance)),
        ),
        make(leader_id, IfccadEntityKind::Leader(Box::new(leader))),
        make(multi_id, IfccadEntityKind::MultiLeader(Box::new(multi))),
    ]);
    validate_ifccad_document(&d).unwrap();
    let text = encode_ifccad_document(&d).unwrap();
    let back = load_ifccad_bytes(text.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(back, d);
    let binary = encode_ifccad_cbor_document(&d).unwrap();
    let back = load_ifccad_cbor_bytes(binary.bytes(), Default::default()).unwrap();
    assert_eq!(back.document(), &d);
    let target = d
        .model
        .entities
        .iter_mut()
        .find(|entity| entity.id() == leader_id)
        .unwrap()
        .as_native_mut()
        .unwrap();
    let IfccadEntityKind::Leader(leader) = &mut target.kind else {
        panic!()
    };
    leader.annotation_entity = Some(multi_id);
    assert!(validate_ifccad_document(&d).is_err());
}
