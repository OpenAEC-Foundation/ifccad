//! Checked, atomic text construction into the existing entity/owner domains.
use crate::{ocdraw::*, text::*};

#[derive(Clone, Debug)]
pub struct TextStyleDefinition {
    pub name: String,
    pub properties: TextStyleProperties,
}

#[derive(Clone, Debug)]
pub struct TextEntityDefinition {
    pub scope_id: u32,
    pub layer_id: u32,
    pub visible: bool,
    pub appearance: EntityAppearance,
    pub style_id: OcdrawTextStyleId,
    pub placement: CoordinateFrame3,
    pub rotation: f64,
    pub backward: bool,
    pub upside_down: bool,
    pub layout: TextLayout,
    pub oblique_angle: f64,
    pub thickness: f64,
    pub content: Vec<TextRun>,
}
impl TextEntityDefinition {
    pub fn new(
        layer_id: u32,
        style_id: OcdrawTextStyleId,
        height: f64,
        content: Vec<TextRun>,
    ) -> Self {
        Self {
            scope_id: 0,
            layer_id,
            style_id,
            visible: true,
            appearance: Default::default(),
            placement: Default::default(),
            rotation: 0.,
            backward: false,
            upside_down: false,
            layout: TextLayout::Anchored {
                horizontal: TextHorizontalAlignment::Left,
                vertical: TextVerticalAlignment::Baseline,
                height,
                width_factor: 1.,
            },
            oblique_angle: 0.,
            thickness: 0.,
            content,
        }
    }
    pub fn in_scope(mut self, scope_id: u32) -> Self {
        self.scope_id = scope_id;
        self
    }
}
#[derive(Clone, Debug)]
pub struct MTextEntityDefinition {
    pub scope_id: u32,
    pub layer_id: u32,
    pub visible: bool,
    pub appearance: EntityAppearance,
    pub style_id: OcdrawTextStyleId,
    pub placement: CoordinateFrame3,
    pub rotation: f64,
    pub backward: bool,
    pub upside_down: bool,
    pub height: f64,
    pub attachment: MTextAttachment,
    pub flow: MTextFlow,
    pub wrap_width: Option<f64>,
    pub columns: Option<MTextColumns>,
    pub background: Option<MTextBackground<DrawingColor>>,
    pub character_format: CharacterFormat<DrawingColor>,
    pub paragraph_format: ParagraphFormat,
    pub content: Vec<MTextParagraph<DrawingColor>>,
}
impl MTextEntityDefinition {
    pub fn new(
        layer_id: u32,
        style_id: OcdrawTextStyleId,
        height: f64,
        content: Vec<MTextParagraph<DrawingColor>>,
    ) -> Self {
        Self {
            scope_id: 0,
            layer_id,
            style_id,
            visible: true,
            appearance: Default::default(),
            placement: Default::default(),
            rotation: 0.,
            backward: false,
            upside_down: false,
            height,
            attachment: Default::default(),
            flow: Default::default(),
            wrap_width: None,
            columns: None,
            background: None,
            character_format: Default::default(),
            paragraph_format: Default::default(),
            content,
        }
    }
    pub fn in_scope(mut self, scope_id: u32) -> Self {
        self.scope_id = scope_id;
        self
    }
}

fn invalid(message: impl Into<String>) -> OcdrawBuildError {
    OcdrawBuildError::Invalid(message.into())
}
impl OcdrawBuilder {
    pub fn add_text_style(
        &mut self,
        d: TextStyleDefinition,
    ) -> Result<OcdrawTextStyleId, OcdrawBuildError> {
        let id = OcdrawTextStyleId(self.next_text_style_id);
        let next = id.0.checked_add(1).ok_or(OcdrawBuildError::IdExhausted)?;
        let mut styles = self.text_styles.clone();
        styles.push(DrawingTextStyle {
            id,
            name: d.name,
            properties: d.properties,
        });
        let errors = super::super::logical::validate_text_parts(&styles, next, &[], &[]);
        if !errors.is_empty() {
            return Err(invalid(format!("{errors:?}")));
        }
        self.text_styles = styles;
        self.next_text_style_id = next;
        Ok(id)
    }
    fn check_text_owner(
        &self,
        scope_id: u32,
        layer_id: u32,
        appearance: &EntityAppearance,
    ) -> Result<(), OcdrawBuildError> {
        if scope_id as usize >= 1 + self.paper_layouts.len() + self.block_definitions.len()
            || layer_id as usize >= self.layers.len()
        {
            return Err(invalid("text owner scope and layer must already exist"));
        }
        if !appearance.line_pattern_scale.is_finite()
            || appearance.line_pattern_scale <= 0.
            || matches!(appearance.line_pattern,AppearanceSelection::Explicit(id) if id.0 as usize >= self.line_patterns.len())
        {
            return Err(invalid("invalid text line pattern reference or scale"));
        }
        Ok(())
    }
    pub fn add_text(&mut self, d: TextEntityDefinition) -> Result<u64, OcdrawBuildError> {
        self.check_text_owner(d.scope_id, d.layer_id, &d.appearance)?;
        let id = self.next_entity_id;
        let next = id.checked_add(1).ok_or(OcdrawBuildError::IdExhausted)?;
        let scope = d.scope_id;
        let t = DrawingTextEntity {
            id,
            layer_id: d.layer_id,
            style_id: d.style_id,
            visible: d.visible,
            appearance: d.appearance,
            placement: d.placement,
            rotation: d.rotation,
            backward: d.backward,
            upside_down: d.upside_down,
            layout: d.layout,
            oblique_angle: d.oblique_angle,
            thickness: d.thickness,
            content: d.content,
        };
        let errors = super::super::logical::validate_text_parts(
            &self.text_styles,
            self.next_text_style_id,
            std::slice::from_ref(&t),
            &[],
        );
        if !errors.is_empty() {
            return Err(invalid(format!("{errors:?}")));
        }
        let style = &self
            .text_styles
            .iter()
            .find(|s| s.id == t.style_id)
            .expect("validated style reference")
            .properties;
        estimate_text_extent(&ResolvedTextExtentInput {
            content: &t.content,
            layout: t.layout,
            placement: t.placement,
            rotation: t.rotation,
            backward: t.backward,
            upside_down: t.upside_down,
            oblique_angle: t.oblique_angle,
            thickness: t.thickness,
            vertical: style.vertical,
        })
        .map_err(|e| invalid(e.to_string()))?;
        self.text_entities.push(t);
        self.scope_entities.entry(scope).or_default().push(id);
        self.next_entity_id = next;
        Ok(id)
    }
    pub fn add_mtext(&mut self, d: MTextEntityDefinition) -> Result<u64, OcdrawBuildError> {
        self.check_text_owner(d.scope_id, d.layer_id, &d.appearance)?;
        let id = self.next_entity_id;
        let next = id.checked_add(1).ok_or(OcdrawBuildError::IdExhausted)?;
        let scope = d.scope_id;
        let t = DrawingMTextEntity {
            id,
            layer_id: d.layer_id,
            style_id: d.style_id,
            visible: d.visible,
            appearance: d.appearance,
            placement: d.placement,
            rotation: d.rotation,
            backward: d.backward,
            upside_down: d.upside_down,
            height: d.height,
            attachment: d.attachment,
            flow: d.flow,
            wrap_width: d.wrap_width,
            columns: d.columns,
            background: d.background,
            character_format: d.character_format,
            paragraph_format: d.paragraph_format,
            content: d.content,
        };
        let errors = super::super::logical::validate_text_parts(
            &self.text_styles,
            self.next_text_style_id,
            &[],
            std::slice::from_ref(&t),
        );
        if !errors.is_empty() {
            return Err(invalid(format!("{errors:?}")));
        }
        let style = &self
            .text_styles
            .iter()
            .find(|s| s.id == t.style_id)
            .expect("validated style reference")
            .properties;
        estimate_mtext_extent(&ResolvedMTextExtentInput {
            content: &t.content,
            style,
            character_format: &t.character_format,
            paragraph_format: &t.paragraph_format,
            height: t.height,
            attachment: t.attachment,
            flow: t.flow,
            wrap_width: t.wrap_width,
            columns: t.columns.as_ref(),
            background: t.background.as_ref(),
            frame_stroke_width: if t.appearance.line_weight == AppearanceSelection::Explicit(0.) {
                Some(0.)
            } else {
                None
            },
            placement: t.placement,
            rotation: t.rotation,
            backward: t.backward,
            upside_down: t.upside_down,
        })
        .map_err(|e| invalid(e.to_string()))?;
        self.mtext_entities.push(t);
        self.scope_entities.entry(scope).or_default().push(id);
        self.next_entity_id = next;
        Ok(id)
    }
}
