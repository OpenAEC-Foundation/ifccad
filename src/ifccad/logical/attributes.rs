//! Stored block attribute definitions and independently placed INSERT children.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadAttributeFlags {
    #[serde(default)]
    pub invisible: bool,
    #[serde(default)]
    pub constant: bool,
    #[serde(default)]
    pub verify: bool,
    #[serde(default)]
    pub preset: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum IfccadAttributeVisibility {
    #[default]
    Normal,
    HideAll,
    ShowAll,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadAttributeMText {
    pub visible: bool,
    pub layer_id: u64,
    pub appearance: IfccadEntityAppearance,
    pub line_pattern_scale: f64,
    pub text: IfccadMText,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadAttributeDefinition {
    pub tag: String,
    pub prompt: String,
    pub flags: IfccadAttributeFlags,
    pub lock_position: bool,
    pub field_length: i32,
    pub text: IfccadText,
    pub m_text: Option<Box<IfccadAttributeMText>>,
    pub annotation: Option<Box<IfccadAnnotation<IfccadAttributeAnnotationContext>>>,
    pub secondary_attributes: Vec<u64>,
    pub fields: Option<IfccadAttributeFieldGraph>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadAttributeValue {
    pub tag: String,
    pub definition_id: Option<u64>,
    pub flags: IfccadAttributeFlags,
    pub lock_position: bool,
    pub field_length: i32,
    pub text: IfccadText,
    pub m_text: Option<Box<IfccadAttributeMText>>,
    pub annotation: Option<Box<IfccadAnnotation<IfccadAttributeAnnotationContext>>>,
    pub secondary_attributes: Vec<u64>,
    pub fields: Option<IfccadAttributeFieldGraph>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadBlockAttribute {
    pub id: u64,
    pub visible: bool,
    pub layer_id: u64,
    pub appearance: IfccadEntityAppearance,
    pub line_pattern_scale: f64,
    pub value: IfccadAttributeValue,
}

pub(super) fn fail(path: &str, message: impl Into<String>) -> IfccadReport {
    crate::ifccad::diagnostics::failure("IFCCAD-ATTRIBUTE-001", path, message)
}

pub(super) fn validate_text(
    d: &IfccadDocument,
    tag: &str,
    text: &IfccadText,
    mtext: Option<&IfccadAttributeMText>,
    annotation: Option<&IfccadAnnotation<IfccadAttributeAnnotationContext>>,
    path: &str,
) -> Result<(), IfccadReport> {
    if tag.is_empty() || tag.chars().any(|c| c == ' ' || c.is_control()) {
        return Err(fail(
            path,
            "attribute tag is empty or contains a space/control",
        ));
    }
    if text.annotation.is_some() {
        return Err(fail(path, "attribute annotation belongs on the attribute"));
    }
    super::text::validate_value(d, super::text::TextValue::Text(text), path)?;
    if let Some(m) = mtext {
        if m.text.annotation.is_some() {
            return Err(fail(path, "embedded annotation belongs on the attribute"));
        }
        common(d, m.layer_id, &m.appearance, m.line_pattern_scale, path)?;
        super::text::validate_value(d, super::text::TextValue::MText(&m.text), path)?;
    }
    if let Some(a) = annotation {
        a.validate_memberships(&d.annotation_scales, false)?;
        for p in a.contexts.iter().filter_map(|c| c.presentation.as_ref()) {
            p.validate(d, mtext.is_some(), path)?;
        }
    }
    Ok(())
}

fn common(
    d: &IfccadDocument,
    layer: u64,
    appearance: &IfccadEntityAppearance,
    scale: f64,
    path: &str,
) -> Result<(), IfccadReport> {
    if !d.layers.iter().any(|l| l.id == layer) {
        return Err(fail(path, "unresolved attribute layer"));
    }
    super::document_validation::entity_appearance(appearance, path)?;
    if let IfccadMode::Explicit(id) = appearance.line_pattern {
        if !d.line_patterns.iter().any(|p| p.id == id) {
            return Err(fail(path, "unresolved attribute line pattern"));
        }
    }
    super::patterns::scale(scale, path)
}

pub(super) fn validate(d: &IfccadDocument) -> Result<(), IfccadReport> {
    let scopes = std::iter::once((None, &d.model.entities))
        .chain(d.paper_layouts.iter().map(|p| (None, &p.entities)))
        .chain(d.blocks.iter().map(|b| (Some(b.id), &b.entities)));
    let mut all = BTreeSet::new();
    let mut definitions = BTreeMap::new();
    let mut attributes = BTreeSet::new();
    let mut children = Vec::new();
    let mut definitions_values = Vec::new();
    for (owner, entities) in scopes {
        for e in entities {
            if !all.insert(e.id()) {
                return Err(fail("/attributes", "duplicate entity identity"));
            }
            if let Some(e) = e.as_native() {
                match &e.kind {
                    IfccadEntityKind::AttributeDefinition(a) => {
                        definitions.insert(e.id, owner);
                        attributes.insert(e.id);
                        definitions_values.push((e.id, a.as_ref()));
                    }
                    IfccadEntityKind::BlockInstance {
                        definition_id,
                        attributes: items,
                        ..
                    } => {
                        for a in items {
                            children.push((*definition_id, a));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    for (_, a) in &children {
        if !all.insert(a.id) || a.id >= d.id_counters.next_entity_id {
            return Err(fail(
                "/attributes",
                "duplicate child identity or invalid entity watermark",
            ));
        }
        attributes.insert(a.id);
    }
    let secondary = |refs: &[u64], path: &str| -> Result<(), IfccadReport> {
        if refs.iter().any(|id| !attributes.contains(id)) {
            return Err(fail(
                path,
                "secondary target is not an attribute in this drawing",
            ));
        }
        Ok(())
    };
    for (id, a) in definitions_values {
        let path = format!("/cad/d{}/e{id}", d.drawing_id);
        validate_text(
            d,
            &a.tag,
            &a.text,
            a.m_text.as_deref(),
            a.annotation.as_deref(),
            &path,
        )?;
        secondary(&a.secondary_attributes, &path)?;
        if let Some(fields) = &a.fields {
            fields.validate(&all, true, &path)?;
        }
    }
    for (block, a) in children {
        let path = format!("/cad/d{}/e{}", d.drawing_id, a.id);
        common(d, a.layer_id, &a.appearance, a.line_pattern_scale, &path)?;
        let v = &a.value;
        validate_text(
            d,
            &v.tag,
            &v.text,
            v.m_text.as_deref(),
            v.annotation.as_deref(),
            &path,
        )?;
        if let Some(id) = v.definition_id {
            if definitions.get(&id) != Some(&Some(block)) {
                return Err(fail(
                    &path,
                    "attribute definition must belong to the INSERT definition",
                ));
            }
        }
        secondary(&v.secondary_attributes, &path)?;
        if let Some(fields) = &v.fields {
            fields.validate(&all, false, &path)?;
        }
    }
    Ok(())
}
