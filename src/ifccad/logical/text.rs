//! IFCCAD-owned text identities, independent of standalone drawing tables.
use super::IfccadColor;
use crate::text::TextStyleProperties;
use crate::text::*;

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadText {
    pub style_id: IfccadTextStyleId,
    pub placement: super::IfccadPlacement,
    pub rotation: f64,
    pub backward: bool,
    pub upside_down: bool,
    pub layout: TextLayout,
    pub oblique_angle: f64,
    pub thickness: f64,
    pub content: Vec<TextRun>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMText {
    pub style_id: IfccadTextStyleId,
    pub placement: super::IfccadPlacement,
    pub rotation: f64,
    pub backward: bool,
    pub upside_down: bool,
    pub height: f64,
    pub attachment: MTextAttachment,
    pub flow: MTextFlow,
    pub wrap_width: Option<f64>,
    pub columns: Option<MTextColumns>,
    pub background: Option<MTextBackground<IfccadColor>>,
    pub character_format: CharacterFormat<IfccadColor>,
    pub paragraph_format: ParagraphFormat,
    pub content: Vec<MTextParagraph<IfccadColor>>,
}

pub(super) fn validate_entity(
    document: &super::IfccadDocument,
    kind: &super::IfccadEntityKind,
    path: &str,
) -> Result<(), super::IfccadReport> {
    let (id, placement, rotation) = match kind {
        super::IfccadEntityKind::Text(t) => (t.style_id, &t.placement, t.rotation),
        super::IfccadEntityKind::MText(t) => (t.style_id, &t.placement, t.rotation),
        _ => return Ok(()),
    };
    let style = document
        .text_styles
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| {
            crate::ifccad::diagnostics::failure(
                "IFCCAD-TEXT-004",
                &format!("{path}/style"),
                "unresolved text style",
            )
        })?;
    placement.coordinate_frame().map_err(|report| {
        crate::ifccad::diagnostics::context(report, "IFCCAD-GEOMETRY-001", path)
    })?;
    let (rule, attribute) = match kind {
        super::IfccadEntityKind::Text(_) => ("IFCCAD-TEXT-002", "ifccad::text"),
        _ => ("IFCCAD-TEXT-003", "ifccad::mText"),
    };
    if !rotation.is_finite() {
        return Err(crate::ifccad::diagnostics::failure(
            rule,
            &format!("{path}/{attribute}/rotation"),
            format!("{path} nonfinite text rotation"),
        ));
    }
    let error = |e: TextValueError| {
        crate::ifccad::diagnostics::failure(rule, &format!("{path}/{attribute}"), e.to_string())
    };
    match kind {
        super::IfccadEntityKind::Text(t) => {
            validate_text_layout(&t.layout).map_err(error)?;
            validate_text_runs(&t.content).map_err(error)?;
            if !t.oblique_angle.is_finite()
                || t.oblique_angle.abs() >= std::f64::consts::FRAC_PI_2
                || !t.thickness.is_finite()
            {
                return Err(crate::ifccad::diagnostics::failure(
                    rule,
                    &format!("{path}/{attribute}"),
                    format!("{path} invalid text shear or thickness"),
                ));
            }
        }
        super::IfccadEntityKind::MText(t) => {
            let color = |c: &IfccadColor| super::document_validation::color(c);
            let context = MTextValueContext {
                nominal_height: t.height,
                wrap_width: t.wrap_width,
                columns: t.columns.as_ref(),
            };
            validate_character_format(&t.character_format, color).map_err(error)?;
            validate_paragraph_format(&t.paragraph_format).map_err(error)?;
            validate_mtext_content_with_paragraph_format(
                &t.content,
                context,
                &t.paragraph_format,
                color,
            )
            .map_err(error)?;
            validate_mtext_effective_formats(
                &t.content,
                context,
                &style.properties,
                &t.character_format,
                &t.paragraph_format,
                color,
            )
            .map_err(error)?;
            if let Some(background) = &t.background {
                validate_background(background, color).map_err(error)?;
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IfccadTextStyleId(pub u64);

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadTextStyle {
    pub id: IfccadTextStyleId,
    pub name: String,
    pub properties: TextStyleProperties,
}

pub(super) fn validate_styles(document: &super::IfccadDocument) -> Result<(), super::IfccadReport> {
    let mut ids = std::collections::BTreeSet::new();
    let mut names = std::collections::BTreeSet::new();
    for style in &document.text_styles {
        let path = format!("/cad/d{}/textStyle/{}", document.drawing_id, style.id.0);
        if !ids.insert(style.id) {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-ID-001",
                &path,
                format!("{path} duplicate text style ID"),
            ));
        }
        if style.name.is_empty()
            || style.name.contains('\0')
            || !names.insert(crate::ocdraw::names::name_key(&style.name))
        {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-TEXT-001",
                &format!("{path}/ifccad::textStyle/name"),
                format!(
                "{path} text style name must be nonempty, NUL-free and unique under case folding"
            ),
            ));
        }
        crate::text::validate_text_style(&style.properties).map_err(|e| {
            crate::ifccad::diagnostics::failure(
                "IFCCAD-TEXT-001",
                &format!("{path}/ifccad::textStyle"),
                e.to_string(),
            )
        })?;
    }
    Ok(())
}
