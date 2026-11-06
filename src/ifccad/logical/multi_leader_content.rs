//! Multileader block content and compact, entity-owned label bindings.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadMultiLeaderBlockConnection {
    Extents,
    BasePoint,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderBlockContent {
    pub definition_id: u64,
    pub transform: IfccadBlockTransform,
    pub color: IfccadMode<IfccadColor>,
    pub connection: IfccadMultiLeaderBlockConnection,
    /// Independent stored column-major matrix; IO does not derive it from transform.
    pub stored_transform: Option<[f64; 16]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderBlockLabel {
    pub key: u64,
    /// A direct ATTDEF identity, independent of the source attribute index or tag.
    pub definition_id: Option<u64>,
    pub source_index: Option<i32>,
    pub width: f64,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadMultiLeaderTextAttachment {
    TopOfTopLine,
    MiddleOfTopLine,
    MiddleOfText,
    MiddleOfBottomLine,
    BottomOfBottomLine,
    BottomLine,
    BottomOfTopLineUnderlineBottomLine,
    BottomOfTopLineUnderlineTopLine,
    BottomOfTopLineUnderlineAll,
    CenterOfText,
    CenterOfTextOverline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadMultiLeaderTextAttachmentPoint {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadMultiLeaderTextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IfccadMultiLeaderTextAttachments {
    pub left: IfccadMultiLeaderTextAttachment,
    pub right: IfccadMultiLeaderTextAttachment,
    pub top: IfccadMultiLeaderTextAttachment,
    pub bottom: IfccadMultiLeaderTextAttachment,
}

/// Stored background intent also exists when fill is disabled. The nested MText
/// keeps its independent structured layout; neither value overwrites the other.
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderTextBackground {
    pub enabled: bool,
    pub mask_enabled: bool,
    pub color: IfccadMode<IfccadColor>,
    pub scale_factor: f64,
    pub opacity: IfccadMode<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderTextState {
    pub boundary_height: f64,
    pub automatic_height: bool,
    pub word_break: bool,
    pub color: IfccadMode<IfccadColor>,
    pub attachments: IfccadMultiLeaderTextAttachments,
    pub attachment_point: IfccadMultiLeaderTextAttachmentPoint,
    pub alignment: IfccadMultiLeaderTextAlignment,
    pub background: IfccadMultiLeaderTextBackground,
    /// Independent CAD rotation value; text placement retains the actual direction.
    /// IO never recomputes this stored value from that direction.
    pub stored_rotation: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderTextContent {
    pub text: IfccadMText,
    pub state: IfccadMultiLeaderTextState,
}

/// Owned feature-control content has no ordinary entity ID or nested annotation
/// collection. Physical ToleranceContent exchange requires a qualified source.
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderToleranceContent {
    pub placement: IfccadPlacement,
    pub dimension_style: IfccadDimensionStyleId,
    pub overrides: IfccadDimensionOverrides,
    pub notation: IfccadToleranceNotation,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct IfccadMultiLeaderStoredContent {
    pub m_text: Option<Box<IfccadMultiLeaderTextContent>>,
    pub block: Option<IfccadMultiLeaderBlockContent>,
    pub tolerance: Option<Box<IfccadMultiLeaderToleranceContent>>,
}

/// One independent base or scale presentation. Content selection belongs to the
/// Multileader settings; dormant payloads remain present in this record.
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderPresentation {
    pub plane: IfccadPlacement,
    pub normal_reversed: bool,
    pub content_base_point: [f64; 3],
    pub scale_factor: f64,
    pub text_height: f64,
    pub arrow_size: f64,
    pub landing_gap: f64,
    pub next_part_id: u64,
    pub groups: Vec<IfccadMultiLeaderGroup>,
    pub content: IfccadMultiLeaderStoredContent,
}

pub(super) fn color(value: &IfccadMode<IfccadColor>, path: &str) -> Result<(), IfccadReport> {
    if let IfccadMode::Explicit(value) = value {
        if !value.is_valid() {
            return Err(invalid(path, "invalid concrete color"));
        }
    }
    Ok(())
}
pub(super) fn nonnegative(value: f64, path: &str) -> Result<(), IfccadReport> {
    if !value.is_finite() || value < 0. {
        Err(invalid(path, "value must be finite and nonnegative"))
    } else {
        Ok(())
    }
}

impl IfccadMultiLeaderTextContent {
    pub fn validate(&self, drawing: &IfccadDocument) -> Result<(), IfccadReport> {
        let path = "/multiLeader/content/mText";
        if self.text.annotation.is_some() {
            return Err(invalid(
                path,
                "owned MText cannot own another annotation collection",
            ));
        }
        super::text::validate_value(drawing, super::text::TextValue::MText(&self.text), path)?;
        nonnegative(
            self.state.boundary_height,
            &format!("{path}/state/boundaryHeight"),
        )?;
        color(&self.state.color, &format!("{path}/state/color"))?;
        let background = &self.state.background;
        if self
            .state
            .stored_rotation
            .is_some_and(|angle| !angle.is_finite())
        {
            return Err(invalid(
                "/multiLeader/content/mText/state/storedRotation",
                "rotation must be finite",
            ));
        }
        color(&background.color, &format!("{path}/state/background/color"))?;
        nonnegative(
            background.scale_factor,
            &format!("{path}/state/background/scaleFactor"),
        )?;
        if let IfccadMode::Explicit(value) = background.opacity {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(invalid(
                    &format!("{path}/state/background/opacity"),
                    "opacity must be finite between zero and one",
                ));
            }
        }
        Ok(())
    }
}

impl IfccadMultiLeaderToleranceContent {
    pub fn validate(&self, drawing: &IfccadDocument) -> Result<(), IfccadReport> {
        self.placement.coordinate_frame()?;
        super::leader::style(
            drawing,
            self.dimension_style,
            &self.overrides,
            "/multiLeader/content/tolerance/style",
        )?;
        self.notation.validate()
    }
}

impl IfccadMultiLeaderPresentation {
    pub fn validate(&self, drawing: &IfccadDocument) -> Result<(), IfccadReport> {
        let path = "/multiLeader/presentation";
        self.plane.coordinate_frame()?;
        if !self.content_base_point.into_iter().all(f64::is_finite) {
            return Err(invalid(
                &format!("{path}/contentBasePoint"),
                "coordinates must be finite",
            ));
        }
        if !self.scale_factor.is_finite() || self.scale_factor <= 0. {
            return Err(invalid(
                &format!("{path}/scaleFactor"),
                "scale factor must be finite and positive",
            ));
        }
        nonnegative(self.text_height, &format!("{path}/textHeight"))?;
        nonnegative(self.arrow_size, &format!("{path}/arrowSize"))?;
        nonnegative(self.landing_gap, &format!("{path}/landingGap"))?;
        validate_ifccad_multi_leader_branches(drawing, &self.groups, self.next_part_id)?;
        if let Some(text) = &self.content.m_text {
            text.validate(drawing)?;
        }
        if let Some(block) = &self.content.block {
            block.validate(drawing)?;
        }
        if let Some(tolerance) = &self.content.tolerance {
            tolerance.validate(drawing)?;
        }
        Ok(())
    }
}

fn invalid(path: &str, detail: &str) -> IfccadReport {
    crate::ifccad::diagnostics::failure("IFCCAD-MULTILEADER-002", path, detail)
}

impl IfccadMultiLeaderBlockContent {
    pub fn validate(&self, drawing: &IfccadDocument) -> Result<(), IfccadReport> {
        super::document_validation::validate_block_reference(
            drawing,
            self.definition_id,
            &self.transform,
            "/multiLeader/content/block",
            "/multiLeader/content/block/transform",
        )?;
        if let IfccadMode::Explicit(color) = &self.color {
            if !color.is_valid() {
                return Err(invalid(
                    "/multiLeader/content/block/color",
                    "invalid concrete color",
                ));
            }
        }
        if self
            .stored_transform
            .is_some_and(|matrix| matrix.into_iter().any(|value| !value.is_finite()))
        {
            return Err(invalid(
                "/multiLeader/content/block/storedTransform",
                "matrix entries must be finite",
            ));
        }
        Ok(())
    }
}

/// `definitions` contains the nominal content block and every explicitly stored
/// base/scale content block. No nested-block or tag lookup expands this set.
pub fn validate_ifccad_multi_leader_labels(
    drawing: &IfccadDocument,
    labels: &[IfccadMultiLeaderBlockLabel],
    next_label_id: u64,
    definitions: &BTreeSet<u64>,
) -> Result<(), IfccadReport> {
    if next_label_id == 0 {
        return Err(invalid(
            "/multiLeader/nextLabelId",
            "watermark must be positive",
        ));
    }
    let blocks = drawing
        .blocks
        .iter()
        .map(|block| (block.id, block))
        .collect::<BTreeMap<_, _>>();
    let mut direct = BTreeSet::new();
    for id in definitions {
        let block = blocks
            .get(id)
            .ok_or_else(|| invalid("/multiLeader/labels", "unresolved content block"))?;
        for entity in block.entities.iter().filter_map(IfccadEntity::as_native) {
            if matches!(entity.kind, IfccadEntityKind::AttributeDefinition(_)) {
                direct.insert(entity.id);
            }
        }
    }
    let mut keys = BTreeSet::new();
    for (index, label) in labels.iter().enumerate() {
        let path = format!("/multiLeader/labels/{index}");
        if label.key == 0 || label.key >= next_label_id || !keys.insert(label.key) {
            return Err(invalid(
                &format!("{path}/key"),
                "duplicate, zero or unallocated label key",
            ));
        }
        if label.definition_id.is_some_and(|id| !direct.contains(&id)) {
            return Err(invalid(
                &format!("{path}/definition"),
                "label must bind a direct ATTDEF of a declared content block",
            ));
        }
        if !label.width.is_finite() || label.width < 0. {
            return Err(invalid(
                &format!("{path}/width"),
                "width must be finite and nonnegative",
            ));
        }
        if label.text.contains('\0') || label.text.contains("%<") {
            return Err(invalid(
                &format!("{path}/text"),
                "NUL or an unbound embedded field is unsupported",
            ));
        }
    }
    Ok(())
}
