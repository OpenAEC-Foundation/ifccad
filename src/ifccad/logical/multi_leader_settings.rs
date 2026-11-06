//! Explicit authoring settings, independent of saved base/scale presentations.
use super::multi_leader_content::{color, nonnegative};
use super::*;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IfccadMultiLeaderStyleId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadMultiLeaderContentKind {
    None,
    MText,
    Block,
    Tolerance,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadMultiLeaderTextAngle {
    ParallelToLastSegment,
    Horizontal,
    ReadableParallel,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadMultiLeaderDrawOrder {
    HeadFirst,
    TailFirst,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadMultiLeaderContentDrawOrder {
    ContentFirst,
    LeadersFirst,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderTextSettings {
    pub style_id: Option<IfccadTextStyleId>,
    pub color: IfccadMode<IfccadColor>,
    pub height: f64,
    pub frame: bool,
    pub attachments: IfccadMultiLeaderTextAttachments,
    pub attachment_direction: IfccadMultiLeaderAttachmentDirection,
    pub angle: IfccadMultiLeaderTextAngle,
    pub alignment: IfccadMultiLeaderTextAlignment,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderBlockSettings {
    pub definition_id: Option<u64>,
    pub color: IfccadMode<IfccadColor>,
    pub connection: IfccadMultiLeaderBlockConnection,
    pub rotation: f64,
    pub scale: [f64; 3],
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderCommonSettings {
    pub line: IfccadMultiLeaderLineSettings,
    pub landing_enabled: bool,
    pub dogleg_enabled: bool,
    pub dogleg_length: f64,
    pub content_kind: IfccadMultiLeaderContentKind,
    pub text: IfccadMultiLeaderTextSettings,
    pub block: IfccadMultiLeaderBlockSettings,
    pub scale_factor: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderSettings {
    pub common: IfccadMultiLeaderCommonSettings,
    pub text_attachment_point: IfccadMultiLeaderTextAttachmentPoint,
    pub text_direction_negative: bool,
    pub extend_leader_to_text: bool,
}
/// Presence of an override is retained even when its value equals its style.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IfccadMultiLeaderProperty {
    PathKind,
    LineColor,
    LinePattern,
    LineWeight,
    LandingEnabled,
    LandingGap,
    DoglegEnabled,
    DoglegLength,
    Arrow,
    ArrowSize,
    ContentKind,
    TextStyle,
    TextLeftAttachment,
    TextAngle,
    TextAlignment,
    TextColor,
    TextHeight,
    TextFrame,
    UseDefaultMText,
    BlockDefinition,
    BlockColor,
    BlockScale,
    BlockRotation,
    BlockConnection,
    ScaleFactor,
    TextRightAttachment,
    TextSwitchAlignment,
    TextAttachmentDirection,
    TextTopAttachment,
    TextBottomAttachment,
}
/// Structured default text has no fabricated placement or nested entity identity.
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderDefaultText {
    pub character_format: crate::text::CharacterFormat<IfccadColor>,
    pub paragraph_format: crate::text::ParagraphFormat,
    pub content: Vec<crate::text::MTextParagraph<IfccadColor>>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderStyle {
    pub id: IfccadMultiLeaderStyleId,
    pub name: String,
    pub description: String,
    pub common: IfccadMultiLeaderCommonSettings,
    pub landing_gap: f64,
    pub default_m_text: IfccadMultiLeaderDefaultText,
    pub text_always_left: bool,
    pub block_scale_enabled: bool,
    pub block_rotation_enabled: bool,
    pub align_space: f64,
    pub break_gap_size: f64,
    pub max_leader_points: u32,
    pub first_segment_angle: f64,
    pub second_segment_angle: f64,
    pub leader_draw_order: IfccadMultiLeaderDrawOrder,
    pub content_draw_order: IfccadMultiLeaderContentDrawOrder,
    pub annotative: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderLegacyArrowOverride {
    pub line_key: u64,
    pub is_default: bool,
    pub arrow: IfccadLeaderArrow,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeader {
    pub style_id: Option<IfccadMultiLeaderStyleId>,
    pub settings: IfccadMultiLeaderSettings,
    pub overridden_properties: BTreeSet<IfccadMultiLeaderProperty>,
    pub block_labels: Vec<IfccadMultiLeaderBlockLabel>,
    pub next_label_id: u64,
    pub legacy_arrow_overrides: Vec<IfccadMultiLeaderLegacyArrowOverride>,
    pub presentation: IfccadMultiLeaderPresentation,
    pub annotation: Option<IfccadAnnotation<IfccadMultiLeaderPresentation>>,
    pub graphics: Option<IfccadLeaderGraphics>,
}

fn identity() -> IfccadPlacement {
    IfccadPlacement {
        origin: [0.; 3],
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    }
}
fn fail(path: &str, message: &str) -> IfccadReport {
    crate::ifccad::diagnostics::failure("IFCCAD-MULTILEADER-003", path, message)
}
fn positive(value: f64, path: &str) -> Result<(), IfccadReport> {
    if value.is_finite() && value > 0. {
        Ok(())
    } else {
        Err(fail(path, "value must be finite and positive"))
    }
}
fn finite(value: f64, path: &str) -> Result<(), IfccadReport> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(fail(path, "value must be finite"))
    }
}

/// Authoring defaults only. Readers require saved fields rather than applying
/// these defaults to absent source data or inventing a Standard resource link.
impl Default for IfccadMultiLeaderCommonSettings {
    fn default() -> Self {
        Self {
            line: IfccadMultiLeaderLineSettings {
                path_kind: IfccadMultiLeaderPathKind::StraightSegments,
                color: IfccadMode::ByBlock,
                line_pattern: IfccadMode::ByLayer,
                line_weight: IfccadLeaderLineWeight::ByLayer,
                arrow: IfccadLeaderArrow::DefaultClosedFilled,
                arrow_size: 2.5,
            },
            landing_enabled: true,
            dogleg_enabled: true,
            dogleg_length: 5.,
            content_kind: IfccadMultiLeaderContentKind::MText,
            text: IfccadMultiLeaderTextSettings {
                style_id: None,
                color: IfccadMode::ByBlock,
                height: 2.5,
                frame: false,
                attachments: IfccadMultiLeaderTextAttachments {
                    left: IfccadMultiLeaderTextAttachment::MiddleOfText,
                    right: IfccadMultiLeaderTextAttachment::MiddleOfText,
                    top: IfccadMultiLeaderTextAttachment::CenterOfText,
                    bottom: IfccadMultiLeaderTextAttachment::CenterOfText,
                },
                attachment_direction: IfccadMultiLeaderAttachmentDirection::Horizontal,
                angle: IfccadMultiLeaderTextAngle::Horizontal,
                alignment: IfccadMultiLeaderTextAlignment::Left,
            },
            block: IfccadMultiLeaderBlockSettings {
                definition_id: None,
                color: IfccadMode::ByBlock,
                connection: IfccadMultiLeaderBlockConnection::Extents,
                rotation: 0.,
                scale: [1.; 3],
            },
            scale_factor: 1.,
        }
    }
}
impl IfccadMultiLeaderStyle {
    pub fn new(id: IfccadMultiLeaderStyleId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            description: String::new(),
            common: Default::default(),
            landing_gap: 0.625,
            default_m_text: IfccadMultiLeaderDefaultText {
                character_format: Default::default(),
                paragraph_format: Default::default(),
                content: vec![Default::default()],
            },
            text_always_left: false,
            block_scale_enabled: true,
            block_rotation_enabled: true,
            align_space: 0.,
            break_gap_size: 0.,
            max_leader_points: 0,
            first_segment_angle: 0.,
            second_segment_angle: 0.,
            leader_draw_order: IfccadMultiLeaderDrawOrder::HeadFirst,
            content_draw_order: IfccadMultiLeaderContentDrawOrder::ContentFirst,
            annotative: false,
        }
    }
}
impl Default for IfccadMultiLeader {
    fn default() -> Self {
        Self::new()
    }
}
impl IfccadMultiLeader {
    pub fn new() -> Self {
        Self {
            style_id: None,
            settings: IfccadMultiLeaderSettings {
                common: Default::default(),
                text_attachment_point: IfccadMultiLeaderTextAttachmentPoint::Center,
                text_direction_negative: false,
                extend_leader_to_text: false,
            },
            overridden_properties: BTreeSet::new(),
            block_labels: vec![],
            next_label_id: 1,
            legacy_arrow_overrides: vec![],
            presentation: IfccadMultiLeaderPresentation {
                plane: identity(),
                normal_reversed: false,
                content_base_point: [0.; 3],
                scale_factor: 1.,
                text_height: 2.5,
                arrow_size: 2.5,
                landing_gap: 0.625,
                next_part_id: 1,
                groups: vec![],
                content: Default::default(),
            },
            annotation: None,
            graphics: None,
        }
    }
}

impl IfccadMultiLeaderCommonSettings {
    pub fn validate(&self, drawing: &IfccadDocument) -> Result<(), IfccadReport> {
        self.line.validate(drawing)?;
        nonnegative(self.dogleg_length, "/multiLeader/settings/doglegLength")?;
        positive(self.scale_factor, "/multiLeader/settings/scaleFactor")?;
        positive(self.text.height, "/multiLeader/settings/text/height")?;
        if self
            .text
            .style_id
            .is_some_and(|id| !drawing.text_styles.iter().any(|style| style.id == id))
        {
            return Err(fail(
                "/multiLeader/settings/text/style",
                "unresolved TextStyle",
            ));
        }
        color(&self.text.color, "/multiLeader/settings/text/color")?;
        color(&self.block.color, "/multiLeader/settings/block/color")?;
        if self
            .block
            .definition_id
            .is_some_and(|id| !drawing.blocks.iter().any(|block| block.id == id))
        {
            return Err(fail(
                "/multiLeader/settings/block/definition",
                "unresolved block definition",
            ));
        }
        finite(self.block.rotation, "/multiLeader/settings/block/rotation")?;
        for value in self.block.scale {
            finite(value, "/multiLeader/settings/block/scale")?;
        }
        Ok(())
    }
}
impl IfccadMultiLeaderStyle {
    pub fn validate_values(&self, drawing: &IfccadDocument) -> Result<(), IfccadReport> {
        self.common.validate(drawing)?;
        if self.name.trim().is_empty()
            || self.name.contains('\0')
            || self.description.contains('\0')
        {
            return Err(fail(
                "/multiLeaderStyle/name",
                "invalid style name or description",
            ));
        }
        nonnegative(self.landing_gap, "/multiLeaderStyle/landingGap")?;
        nonnegative(self.align_space, "/multiLeaderStyle/alignSpace")?;
        nonnegative(self.break_gap_size, "/multiLeaderStyle/breakGapSize")?;
        finite(
            self.first_segment_angle,
            "/multiLeaderStyle/firstSegmentAngle",
        )?;
        finite(
            self.second_segment_angle,
            "/multiLeaderStyle/secondSegmentAngle",
        )?;
        use crate::text::*;
        let valid_color = |value: &IfccadColor| value.is_valid();
        let error =
            |value: TextValueError| fail("/multiLeaderStyle/defaultMText", &value.to_string());
        validate_character_format(&self.default_m_text.character_format, valid_color)
            .map_err(error)?;
        validate_paragraph_format(&self.default_m_text.paragraph_format).map_err(error)?;
        let context = MTextValueContext {
            nominal_height: self.common.text.height,
            wrap_width: None,
            columns: None,
        };
        validate_mtext_content_with_paragraph_format(
            &self.default_m_text.content,
            context,
            &self.default_m_text.paragraph_format,
            valid_color,
        )
        .map_err(error)?;
        if let Some(style) = self
            .common
            .text
            .style_id
            .and_then(|id| drawing.text_styles.iter().find(|style| style.id == id))
        {
            validate_mtext_effective_formats(
                &self.default_m_text.content,
                context,
                &style.properties,
                &self.default_m_text.character_format,
                &self.default_m_text.paragraph_format,
                valid_color,
            )
            .map_err(error)?;
        }
        Ok(())
    }
}
pub fn validate_ifccad_multi_leader_styles(
    drawing: &IfccadDocument,
    styles: &[IfccadMultiLeaderStyle],
    next_style_id: u64,
) -> Result<(), IfccadReport> {
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    if next_style_id == 0 {
        return Err(fail(
            "/multiLeaderStyles/nextId",
            "watermark must be positive",
        ));
    }
    for style in styles {
        if style.id.0 >= next_style_id
            || !ids.insert(style.id)
            || !names.insert(crate::names::name_key(&style.name))
        {
            return Err(fail(
                "/multiLeaderStyles",
                "duplicate name/identity or unallocated style ID",
            ));
        }
        style.validate_values(drawing)?;
    }
    Ok(())
}
impl IfccadMultiLeader {
    /// Value/resource checks only; ordinary entity ownership is checked by the
    /// drawing graph when these values are attached as native entity kinds.
    pub fn validate_values(
        &self,
        drawing: &IfccadDocument,
        styles: &[IfccadMultiLeaderStyle],
    ) -> Result<(), IfccadReport> {
        if self
            .style_id
            .is_some_and(|id| styles.iter().filter(|style| style.id == id).count() != 1)
        {
            return Err(fail(
                "/multiLeader/style",
                "unresolved or ambiguous MultiLeaderStyle",
            ));
        }
        self.settings.common.validate(drawing)?;
        self.presentation.validate(drawing)?;
        let mut definitions = BTreeSet::new();
        if let Some(id) = self.settings.common.block.definition_id {
            definitions.insert(id);
        }
        if let Some(block) = &self.presentation.content.block {
            definitions.insert(block.definition_id);
        }
        if let Some(annotation) = &self.annotation {
            annotation.validate_memberships(&drawing.annotation_scales, true)?;
            for context in &annotation.contexts {
                if let Some(presentation) = &context.presentation {
                    presentation.validate(drawing)?;
                    if let Some(block) = &presentation.content.block {
                        definitions.insert(block.definition_id);
                    }
                }
            }
        }
        validate_ifccad_multi_leader_labels(
            drawing,
            &self.block_labels,
            self.next_label_id,
            &definitions,
        )?;
        let lines = self
            .presentation
            .groups
            .iter()
            .flat_map(|group| &group.lines)
            .map(|line| line.key)
            .collect::<BTreeSet<_>>();
        for item in &self.legacy_arrow_overrides {
            if !lines.contains(&item.line_key) {
                return Err(fail(
                    "/multiLeader/legacyArrowOverrides",
                    "unresolved base line key",
                ));
            }
            if let IfccadLeaderArrow::Symbol { definition_id } = item.arrow {
                if !drawing.blocks.iter().any(|block| block.id == definition_id) {
                    return Err(fail(
                        "/multiLeader/legacyArrowOverrides",
                        "unresolved arrow block",
                    ));
                }
            }
        }
        if let Some(graphics) = &self.graphics {
            graphics.validate(drawing)?;
        }
        Ok(())
    }
}
