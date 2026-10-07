//! One semantic text validator for constructed and decoded OCDraw documents.
use super::{field_validation::logical_error, *};
use crate::text::*;
use std::collections::{BTreeMap, BTreeSet};

fn values(errors: &mut Vec<LogicalError>, base: &str, result: Result<(), TextValueError>) {
    if let Err(error) = result {
        errors.push(logical_error(
            "TEXT_VALUE",
            format!("{base}{}", error.path),
            error.message,
        ));
    }
}

pub(crate) fn validate_text(doc: &OcdrawDocument) -> Vec<LogicalError> {
    validate_text_parts(
        &doc.text_styles,
        doc.next_text_style_id,
        &doc.text_entities,
        &doc.mtext_entities,
    )
}

pub(crate) fn validate_text_parts(
    text_styles: &[DrawingTextStyle],
    next_text_style_id: u32,
    text_entities: &[DrawingTextEntity],
    mtext_entities: &[DrawingMTextEntity],
) -> Vec<LogicalError> {
    let mut errors = Vec::new();
    let mut styles = BTreeMap::new();
    let mut names = BTreeSet::new();
    for (i, style) in text_styles.iter().enumerate() {
        let path = format!("/textStyles/{i}");
        if styles.insert(style.id, &style.properties).is_some() {
            errors.push(logical_error(
                "TEXT_STYLE_ID",
                format!("{path}/id"),
                "duplicate text style identity",
            ));
        }
        if style.name.is_empty()
            || style.name.contains('\0')
            || !names.insert(crate::ocdraw::names::name_key(&style.name))
        {
            errors.push(logical_error(
                "TEXT_STYLE_NAME",
                format!("{path}/name"),
                "text style names must be nonempty, NUL-free and unique under case folding",
            ));
        }
        values(&mut errors, &path, validate_text_style(&style.properties));
    }
    if next_text_style_id == 0 || styles.keys().any(|id| id.0 >= next_text_style_id) {
        errors.push(logical_error(
            "TEXT_STYLE_WATERMARK",
            "/header/nextTextStyleId",
            "text style watermark must be positive and exceed all live IDs",
        ));
    }
    for (id, style_id, appearance, rotation) in text_entities
        .iter()
        .map(|e| (e.id, e.style_id, &e.appearance, e.rotation))
        .chain(
            mtext_entities
                .iter()
                .map(|e| (e.id, e.style_id, &e.appearance, e.rotation)),
        )
    {
        if !styles.contains_key(&style_id) {
            errors.push(logical_error(
                "TEXT_STYLE_REFERENCE",
                format!("/entities/{id}/styleId"),
                "text style reference has no drawing-local target",
            ));
        }
        if !super::field_validation::appearance(appearance) {
            errors.push(logical_error(
                "ENTITY_APPEARANCE",
                format!("/entities/{id}/appearance"),
                "invalid native text appearance",
            ));
        }
        if !rotation.is_finite() {
            errors.push(logical_error(
                "TEXT_VALUE",
                format!("/entities/{id}/rotation"),
                "rotation must be finite",
            ));
        }
    }
    for (i, text) in text_entities.iter().enumerate() {
        let path = format!("/textEntities/{i}");
        values(&mut errors, &path, validate_text_layout(&text.layout));
        values(&mut errors, &path, validate_text_runs(&text.content));
        if !text.oblique_angle.is_finite()
            || text.oblique_angle.abs() >= std::f64::consts::FRAC_PI_2
            || !text.thickness.is_finite()
        {
            errors.push(logical_error("TEXT_VALUE", path, "text shear and thickness must be finite; shear is strictly between minus and plus pi/2"));
        }
    }
    for (i, text) in mtext_entities.iter().enumerate() {
        let path = format!("/mTextEntities/{i}");
        let context = MTextValueContext {
            nominal_height: text.height,
            wrap_width: text.wrap_width,
            columns: text.columns.as_ref(),
        };
        values(
            &mut errors,
            &path,
            validate_character_format(&text.character_format, super::field_validation::color),
        );
        values(
            &mut errors,
            &path,
            validate_paragraph_format(&text.paragraph_format),
        );
        values(
            &mut errors,
            &path,
            validate_mtext_content_with_paragraph_format(
                &text.content,
                context,
                &text.paragraph_format,
                super::field_validation::color,
            ),
        );
        if let Some(style) = styles.get(&text.style_id) {
            values(
                &mut errors,
                &path,
                validate_mtext_effective_formats(
                    &text.content,
                    context,
                    style,
                    &text.character_format,
                    &text.paragraph_format,
                    super::field_validation::color,
                ),
            );
        }
        if let Some(background) = &text.background {
            values(
                &mut errors,
                &format!("{path}/background"),
                validate_background(background, super::field_validation::color),
            );
        }
    }
    errors
}
