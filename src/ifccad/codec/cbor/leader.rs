//! Leader-family value records. Entity dispatch and revision activation are separate.
use super::*;
use std::collections::BTreeSet;
record!(IfccadLeader;
0 => placement: IfccadPlacement = None,
1 => path: IfccadLeaderPath = None,
2 => annotation_kind: IfccadLeaderAnnotationKind = None,
3 => annotation_entity: Option<u64> = Some(None),
4 => dimension_style: IfccadDimensionStyleId = None,
5 => overrides: IfccadDimensionOverrides = None,
6 => arrow: Option<IfccadLeaderArrow> = Some(None),
7 => arrow_enabled: bool = None,
8 => hook_enabled: bool = None,
9 => hook_direction: IfccadLeaderHookDirection = None,
10 => annotation_box: Option<[f64; 2]> = Some(None),
11 => horizontal_direction: [f64; 3] = None,
12 => block_offset: [f64; 3] = None,
13 => annotation_offset: [f64; 3] = None,
14 => origin: [f64; 3] = None,
15 => annotation: Option<IfccadAnnotation<IfccadLeaderAnnotationContext>> = Some(None),
16 => graphics: Option<IfccadLeaderGraphics> = Some(None),
);
record!(IfccadLeaderAnnotationContext;
0 => points: Vec<[f64; 3]> = None,
1 => horizontal_direction: [f64; 3] = None,
2 => annotation_enabled: bool = None,
3 => insertion_offset: [f64; 3] = None,
4 => endpoint_projection: [f64; 3] = None,
);
record!(IfccadTolerance;
0 => placement: IfccadPlacement = None,
1 => dimension_style: IfccadDimensionStyleId = None,
2 => overrides: IfccadDimensionOverrides = None,
3 => notation: IfccadToleranceNotation = None,
4 => annotation: Option<IfccadAnnotation<IfccadToleranceAnnotationContext>> = Some(None),
5 => graphics: Option<IfccadLeaderGraphics> = Some(None),
);
record!(IfccadToleranceAnnotationContext;
0 => location: [f64; 3] = None,
1 => horizontal_direction: [f64; 3] = None,
);
record!(IfccadMultiLeaderLineSettings;
0 => path_kind: IfccadMultiLeaderPathKind = None,
1 => color: IfccadMode<IfccadColor> = None,
2 => line_pattern: IfccadMode<IfccadLinePatternId> = None,
3 => line_weight: IfccadLeaderLineWeight = None,
4 => arrow: IfccadLeaderArrow = None,
5 => arrow_size: f64 = None,
);
record!(IfccadLeaderBreakInterval;
0 => start: [f64; 3] = None,
1 => end: [f64; 3] = None,
);
record!(IfccadMultiLeaderSegmentBreaks;
0 => segment_index: u32 = None,
1 => intervals: Vec<IfccadLeaderBreakInterval> = None,
);
record!(IfccadMultiLeaderLine;
0 => key: u64 = None,
1 => source_index: Option<i32> = Some(None),
2 => settings: IfccadMultiLeaderLineSettings = None,
3 => overridden_properties: BTreeSet<IfccadMultiLeaderLineProperty> = None,
4 => vertices: Vec<[f64; 3]> = None,
5 => connection: IfccadMultiLeaderConnection = None,
6 => breaks: Vec<IfccadMultiLeaderSegmentBreaks> = None,
);
record!(IfccadMultiLeaderGroup;
0 => key: u64 = None,
1 => source_index: Option<i32> = Some(None),
2 => has_connection_point: bool = None,
3 => connection_point: [f64; 3] = None,
4 => has_dogleg_direction: bool = None,
5 => dogleg_direction: [f64; 3] = None,
6 => dogleg_length: f64 = None,
7 => text_attachment_direction: IfccadMultiLeaderAttachmentDirection = None,
8 => lines: Vec<IfccadMultiLeaderLine> = None,
9 => breaks: Vec<IfccadLeaderBreakInterval> = None,
);
record!(IfccadMultiLeaderBlockContent;
0 => definition_id: u64 = None,
1 => transform: IfccadBlockTransform = None,
2 => color: IfccadMode<IfccadColor> = None,
3 => connection: IfccadMultiLeaderBlockConnection = None,
4 => stored_transform: Option<[f64; 16]> = Some(None),
);
record!(IfccadMultiLeaderBlockLabel;
0 => key: u64 = None,
1 => definition_id: Option<u64> = Some(None),
2 => source_index: Option<i32> = Some(None),
3 => width: f64 = None,
4 => text: String = None,
);
record!(IfccadMultiLeaderTextAttachments;
0 => left: IfccadMultiLeaderTextAttachment = None,
1 => right: IfccadMultiLeaderTextAttachment = None,
2 => top: IfccadMultiLeaderTextAttachment = None,
3 => bottom: IfccadMultiLeaderTextAttachment = None,
);
record!(IfccadMultiLeaderTextBackground;
0 => enabled: bool = None,
1 => mask_enabled: bool = None,
2 => color: IfccadMode<IfccadColor> = None,
3 => scale_factor: f64 = None,
4 => opacity: IfccadMode<f64> = None,
);
record!(IfccadMultiLeaderTextState;
0 => boundary_height: f64 = None,
1 => automatic_height: bool = None,
2 => word_break: bool = None,
3 => color: IfccadMode<IfccadColor> = None,
4 => attachments: IfccadMultiLeaderTextAttachments = None,
5 => attachment_point: IfccadMultiLeaderTextAttachmentPoint = None,
6 => alignment: IfccadMultiLeaderTextAlignment = None,
7 => background: IfccadMultiLeaderTextBackground = None,
8 => stored_rotation: Option<f64> = Some(None),
);
record!(IfccadMultiLeaderTextContent;
0 => text: IfccadMText = None,
1 => state: IfccadMultiLeaderTextState = None,
);
record!(IfccadMultiLeaderToleranceContent;
0 => placement: IfccadPlacement = None,
1 => dimension_style: IfccadDimensionStyleId = None,
2 => overrides: IfccadDimensionOverrides = None,
3 => notation: IfccadToleranceNotation = None,
);
record!(IfccadMultiLeaderStoredContent;
0 => m_text: Option<Box<IfccadMultiLeaderTextContent>> = Some(None),
1 => block: Option<IfccadMultiLeaderBlockContent> = Some(None),
2 => tolerance: Option<Box<IfccadMultiLeaderToleranceContent>> = Some(None),
);
record!(IfccadMultiLeaderPresentation;
0 => plane: IfccadPlacement = None,
1 => normal_reversed: bool = None,
2 => content_base_point: [f64; 3] = None,
3 => scale_factor: f64 = None,
4 => text_height: f64 = None,
5 => arrow_size: f64 = None,
6 => landing_gap: f64 = None,
7 => next_part_id: u64 = None,
8 => groups: Vec<IfccadMultiLeaderGroup> = None,
9 => content: IfccadMultiLeaderStoredContent = None,
);
record!(IfccadMultiLeaderTextSettings;
0 => style_id: Option<IfccadTextStyleId> = Some(None),
1 => color: IfccadMode<IfccadColor> = None,
2 => height: f64 = None,
3 => frame: bool = None,
4 => attachments: IfccadMultiLeaderTextAttachments = None,
5 => attachment_direction: IfccadMultiLeaderAttachmentDirection = None,
6 => angle: IfccadMultiLeaderTextAngle = None,
7 => alignment: IfccadMultiLeaderTextAlignment = None,
);
record!(IfccadMultiLeaderBlockSettings;
0 => definition_id: Option<u64> = Some(None),
1 => color: IfccadMode<IfccadColor> = None,
2 => connection: IfccadMultiLeaderBlockConnection = None,
3 => rotation: f64 = None,
4 => scale: [f64; 3] = None,
);
record!(IfccadMultiLeaderCommonSettings;
0 => line: IfccadMultiLeaderLineSettings = None,
1 => landing_enabled: bool = None,
2 => dogleg_enabled: bool = None,
3 => dogleg_length: f64 = None,
4 => content_kind: IfccadMultiLeaderContentKind = None,
5 => text: IfccadMultiLeaderTextSettings = None,
6 => block: IfccadMultiLeaderBlockSettings = None,
7 => scale_factor: f64 = None,
);
record!(IfccadMultiLeaderSettings;
0 => common: IfccadMultiLeaderCommonSettings = None,
1 => text_attachment_point: IfccadMultiLeaderTextAttachmentPoint = None,
2 => text_direction_negative: bool = None,
3 => extend_leader_to_text: bool = None,
);
record!(IfccadMultiLeaderDefaultText;
0 => character_format: crate::text::CharacterFormat<IfccadColor> = None,
1 => paragraph_format: crate::text::ParagraphFormat = None,
2 => content: Vec<crate::text::MTextParagraph<IfccadColor>> = None,
);
record!(IfccadMultiLeaderStyle;
0 => id: IfccadMultiLeaderStyleId = None,
1 => name: String = None,
2 => description: String = None,
3 => common: IfccadMultiLeaderCommonSettings = None,
4 => landing_gap: f64 = None,
5 => default_m_text: IfccadMultiLeaderDefaultText = None,
6 => text_always_left: bool = None,
7 => block_scale_enabled: bool = None,
8 => block_rotation_enabled: bool = None,
9 => align_space: f64 = None,
10 => break_gap_size: f64 = None,
11 => max_leader_points: u32 = None,
12 => first_segment_angle: f64 = None,
13 => second_segment_angle: f64 = None,
14 => leader_draw_order: IfccadMultiLeaderDrawOrder = None,
15 => content_draw_order: IfccadMultiLeaderContentDrawOrder = None,
16 => annotative: bool = None,
);
record!(IfccadMultiLeaderLegacyArrowOverride;
0 => line_key: u64 = None,
1 => is_default: bool = None,
2 => arrow: IfccadLeaderArrow = None,
);
record!(IfccadMultiLeader;
0 => style_id: Option<IfccadMultiLeaderStyleId> = Some(None),
1 => settings: IfccadMultiLeaderSettings = None,
2 => overridden_properties: BTreeSet<IfccadMultiLeaderProperty> = None,
3 => block_labels: Vec<IfccadMultiLeaderBlockLabel> = None,
4 => next_label_id: u64 = None,
5 => legacy_arrow_overrides: Vec<IfccadMultiLeaderLegacyArrowOverride> = None,
6 => presentation: IfccadMultiLeaderPresentation = None,
7 => annotation: Option<IfccadAnnotation<IfccadMultiLeaderPresentation>> = Some(None),
8 => graphics: Option<IfccadLeaderGraphics> = Some(None),
);
impl CborValue for IfccadLeaderPath {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "StraightSegments"), (1, "SplineThroughVertices")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::StraightSegments(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::SplineThroughVertices(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: Vec<[f64; 3]> = b.take(0, None)?;
                b.finish()?;
                Self::StraightSegments(value_0)
            }
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: Vec<[f64; 3]> = b.take(0, None)?;
                b.finish()?;
                Self::SplineThroughVertices(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StraightSegments(value_0), Self::StraightSegments(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::SplineThroughVertices(value_0), Self::SplineThroughVertices(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            _ => false,
        }
    }
}
impl CborValue for IfccadLeaderArrow {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "DefaultClosedFilled"), (1, "None"), (2, "Symbol")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::DefaultClosedFilled => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::None => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::Symbol { definition_id } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                w.map(u64::from(definition_id.present()))?;
                if definition_id.present() {
                    w.u64(0)?;
                    definition_id.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => Self::DefaultClosedFilled,
            1 => Self::None,
            2 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "definition_id")])?;
                let definition_id: u64 = b.take(0, None)?;
                b.finish()?;
                Self::Symbol { definition_id }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::DefaultClosedFilled, Self::DefaultClosedFilled) => true,
            (Self::None, Self::None) => true,
            (
                Self::Symbol { definition_id },
                Self::Symbol {
                    definition_id: definition_id_other,
                },
            ) => definition_id.exact(definition_id_other),
            _ => false,
        }
    }
}
impl CborValue for IfccadToleranceNotation {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "CadFeatureControlTextV1")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::CadFeatureControlTextV1 { code } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                w.map(u64::from(code.present()))?;
                if code.present() {
                    w.u64(0)?;
                    code.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "code")])?;
                let code: String = b.take(0, None)?;
                b.finish()?;
                Self::CadFeatureControlTextV1 { code }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::CadFeatureControlTextV1 { code },
                Self::CadFeatureControlTextV1 { code: code_other },
            ) => code.exact(code_other),
        }
    }
}
impl CborValue for IfccadLeaderLineWeight {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "ByLayer"),
            (1, "ByBlock"),
            (2, "DrawingDefault"),
            (3, "Explicit"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::ByLayer => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::ByBlock => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::DrawingDefault => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(2)?;
                Ok(())
            }
            Self::Explicit(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(3)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => Self::ByLayer,
            1 => Self::ByBlock,
            2 => Self::DrawingDefault,
            3 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Explicit(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ByLayer, Self::ByLayer) => true,
            (Self::ByBlock, Self::ByBlock) => true,
            (Self::DrawingDefault, Self::DrawingDefault) => true,
            (Self::Explicit(value_0), Self::Explicit(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            _ => false,
        }
    }
}
unit_enum!(IfccadLeaderAnnotationKind;0=>None,1=>MText,2=>Block,3=>Tolerance);
unit_enum!(IfccadLeaderHookDirection;0=>OppositeHorizontal,1=>AlongHorizontal);
unit_enum!(IfccadMultiLeaderPathKind;0=>Invisible,1=>StraightSegments,2=>SplineThroughVertices);
unit_enum!(IfccadMultiLeaderLineProperty;0=>PathKind,1=>Color,2=>LinePattern,3=>LineWeight,4=>Arrow,5=>ArrowSize);
unit_enum!(IfccadMultiLeaderConnection;0=>GroupConnection,1=>ExplicitLastVertex);
unit_enum!(IfccadMultiLeaderAttachmentDirection;0=>Horizontal,1=>Vertical);
unit_enum!(IfccadMultiLeaderBlockConnection;0=>Extents,1=>BasePoint);
unit_enum!(IfccadMultiLeaderTextAttachment;0=>TopOfTopLine,1=>MiddleOfTopLine,2=>MiddleOfText,3=>MiddleOfBottomLine,4=>BottomOfBottomLine,5=>BottomLine,6=>BottomOfTopLineUnderlineBottomLine,7=>BottomOfTopLineUnderlineTopLine,8=>BottomOfTopLineUnderlineAll,9=>CenterOfText,10=>CenterOfTextOverline);
unit_enum!(IfccadMultiLeaderTextAttachmentPoint;0=>Left,1=>Center,2=>Right);
unit_enum!(IfccadMultiLeaderTextAlignment;0=>Left,1=>Center,2=>Right);
unit_enum!(IfccadMultiLeaderContentKind;0=>None,1=>MText,2=>Block,3=>Tolerance);
unit_enum!(IfccadMultiLeaderTextAngle;0=>ParallelToLastSegment,1=>Horizontal,2=>ReadableParallel);
unit_enum!(IfccadMultiLeaderDrawOrder;0=>HeadFirst,1=>TailFirst);
unit_enum!(IfccadMultiLeaderContentDrawOrder;0=>ContentFirst,1=>LeadersFirst);
unit_enum!(IfccadMultiLeaderProperty;0=>PathKind,1=>LineColor,2=>LinePattern,3=>LineWeight,4=>LandingEnabled,5=>LandingGap,6=>DoglegEnabled,7=>DoglegLength,8=>Arrow,9=>ArrowSize,10=>ContentKind,11=>TextStyle,12=>TextLeftAttachment,13=>TextAngle,14=>TextAlignment,15=>TextColor,16=>TextHeight,17=>TextFrame,18=>UseDefaultMText,19=>BlockDefinition,20=>BlockColor,21=>BlockScale,22=>BlockRotation,23=>BlockConnection,24=>ScaleFactor,25=>TextRightAttachment,26=>TextSwitchAlignment,27=>TextAttachmentDirection,28=>TextTopAttachment,29=>TextBottomAttachment);

impl CborValue for IfccadMultiLeaderStyleId {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        self.0.encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        Ok(Self(u64::decode(r)?))
    }
    fn exact(&self, other: &Self) -> bool {
        self == other
    }
}
