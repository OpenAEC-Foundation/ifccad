//! Closed IFCCAD text values; colours retain concrete RGB and optional identity.
use crate::ifccad::IfccadColor;
use crate::text::*;
use serde_json::{json, Value};

macro_rules! tokens {
    ($encode:ident, $decode:ident, $type:ty, {$($variant:path => $token:literal),+}) => {
        pub(super) fn $encode(v: $type) -> Value { json!(match v { $($variant => $token),+ }) }
        pub(super) fn $decode(v: &Value) -> Option<$type> { Some(match v.as_str()? { $($token => $variant),+, _ => return None }) }
    };
}
tokens!(position, read_position, TextPosition, {TextPosition::Bottom=>"bottom",TextPosition::Center=>"center",TextPosition::Top=>"top"});
tokens!(paragraph_alignment, read_paragraph_alignment, TextParagraphAlignment, {TextParagraphAlignment::Left=>"left",TextParagraphAlignment::Center=>"center",TextParagraphAlignment::Right=>"right",TextParagraphAlignment::Justified=>"justified",TextParagraphAlignment::Distributed=>"distributed"});
tokens!(stack_kind, read_stack_kind, TextStackKind, {TextStackKind::Fraction=>"fraction",TextStackKind::DiagonalFraction=>"diagonalFraction",TextStackKind::Tolerance=>"tolerance",TextStackKind::DecimalTolerance=>"decimalTolerance"});
tokens!(stack_alignment, read_stack_alignment, TextStackAlignment, {TextStackAlignment::Top=>"top",TextStackAlignment::Center=>"center",TextStackAlignment::Bottom=>"bottom"});
tokens!(flow, read_flow, MTextFlow, {MTextFlow::Horizontal=>"horizontal",MTextFlow::Vertical=>"vertical",MTextFlow::ByStyle=>"byStyle"});
tokens!(attachment, read_attachment, MTextAttachment, {MTextAttachment::TopLeft=>"topLeft",MTextAttachment::TopCenter=>"topCenter",MTextAttachment::TopRight=>"topRight",MTextAttachment::MiddleLeft=>"middleLeft",MTextAttachment::MiddleCenter=>"middleCenter",MTextAttachment::MiddleRight=>"middleRight",MTextAttachment::BottomLeft=>"bottomLeft",MTextAttachment::BottomCenter=>"bottomCenter",MTextAttachment::BottomRight=>"bottomRight"});

pub(super) fn optional<T>(
    v: Option<&Value>,
    parse: impl FnOnce(&Value) -> Option<T>,
) -> Option<Option<T>> {
    match v {
        None => Some(None),
        Some(v) => parse(v).map(Some),
    }
}
fn scalar_char(v: &Value) -> Option<char> {
    let mut chars = v.as_str()?.chars();
    let first = chars.next()?;
    chars.next().is_none().then_some(first)
}

pub(super) fn font(v: &FontRequest) -> Value {
    let mut row = json!({});
    macro_rules! fields { ($($field:ident=>$key:literal),*) => { $(if let Some(v)=&v.$field { row[$key]=json!(v); })* }; }
    fields!(family=>"family",cad_font_name=>"cadFontName",big_font_name=>"bigFontName",bold=>"bold",italic=>"italic",charset=>"charset",pitch=>"pitch");
    row
}
pub(super) fn read_font(v: &Value) -> Option<FontRequest> {
    Some(FontRequest {
        family: optional(v.get("family"), |v| v.as_str().map(str::to_owned))?,
        cad_font_name: optional(v.get("cadFontName"), |v| v.as_str().map(str::to_owned))?,
        big_font_name: optional(v.get("bigFontName"), |v| v.as_str().map(str::to_owned))?,
        bold: optional(v.get("bold"), Value::as_bool)?,
        italic: optional(v.get("italic"), Value::as_bool)?,
        charset: optional(v.get("charset"), |v| u16::try_from(v.as_u64()?).ok())?,
        pitch: optional(v.get("pitch"), |v| u8::try_from(v.as_u64()?).ok())?,
    })
}
fn height(v: TextHeight) -> Value {
    match v {
        TextHeight::Absolute { distance } => json!({"kind":"absolute","distance":distance}),
        TextHeight::Relative { factor } => json!({"kind":"relative","factor":factor}),
    }
}
fn read_height(v: &Value) -> Option<TextHeight> {
    Some(match v.get("kind")?.as_str()? {
        "absolute" => TextHeight::Absolute {
            distance: v.get("distance")?.as_f64()?,
        },
        "relative" => TextHeight::Relative {
            factor: v.get("factor")?.as_f64()?,
        },
        _ => return None,
    })
}
fn text_color(v: &TextColor<IfccadColor>) -> Value {
    match v {
        TextColor::Entity => json!({"kind":"entity"}),
        TextColor::ByLayer => json!({"kind":"byLayer"}),
        TextColor::ByBlock => json!({"kind":"byBlock"}),
        TextColor::Explicit(c) => json!({"kind":"explicit","color":json!(c)}),
    }
}
fn read_text_color(v: &Value) -> Option<TextColor<IfccadColor>> {
    Some(match v.get("kind")?.as_str()? {
        "entity" => TextColor::Entity,
        "byLayer" => TextColor::ByLayer,
        "byBlock" => TextColor::ByBlock,
        "explicit" => TextColor::Explicit(super::presentation::read_color(v.get("color")?)?),
        _ => return None,
    })
}

pub(super) fn character(v: &CharacterFormat<IfccadColor>) -> Value {
    let mut row = json!({});
    macro_rules! fields { ($($field:ident=>$key:literal),*) => { $(if let Some(v)=&v.$field { row[$key]=json!(v); })* }; }
    fields!(width_factor=>"widthFactor",tracking=>"tracking",oblique_angle=>"obliqueAngle",underline=>"underline",overline=>"overline",strike_through=>"strikeThrough");
    if let Some(v) = &v.font {
        row["font"] = font(v);
    }
    if let Some(v) = &v.color {
        row["color"] = text_color(v);
    }
    if let Some(v) = v.height {
        row["height"] = height(v);
    }
    if let Some(v) = v.position {
        row["position"] = position(v);
    }
    row
}
pub(super) fn read_character(v: &Value) -> Option<CharacterFormat<IfccadColor>> {
    Some(CharacterFormat {
        font: optional(v.get("font"), read_font)?,
        color: optional(v.get("color"), read_text_color)?,
        height: optional(v.get("height"), read_height)?,
        position: optional(v.get("position"), read_position)?,
        width_factor: optional(v.get("widthFactor"), Value::as_f64)?,
        tracking: optional(v.get("tracking"), Value::as_f64)?,
        oblique_angle: optional(v.get("obliqueAngle"), Value::as_f64)?,
        underline: optional(v.get("underline"), Value::as_bool)?,
        overline: optional(v.get("overline"), Value::as_bool)?,
        strike_through: optional(v.get("strikeThrough"), Value::as_bool)?,
    })
}
fn spacing(v: TextLineSpacing) -> Value {
    match v {
        TextLineSpacing::Exact { distance } => json!({"kind":"exact","distance":distance}),
        TextLineSpacing::AtLeast { distance } => json!({"kind":"atLeast","distance":distance}),
        TextLineSpacing::Multiple { factor } => json!({"kind":"multiple","factor":factor}),
    }
}
fn read_spacing(v: &Value) -> Option<TextLineSpacing> {
    Some(match v.get("kind")?.as_str()? {
        "exact" => TextLineSpacing::Exact {
            distance: v.get("distance")?.as_f64()?,
        },
        "atLeast" => TextLineSpacing::AtLeast {
            distance: v.get("distance")?.as_f64()?,
        },
        "multiple" => TextLineSpacing::Multiple {
            factor: v.get("factor")?.as_f64()?,
        },
        _ => return None,
    })
}
fn tab(v: &TextTabStop) -> Value {
    let mut row = json!({"positionFactor":v.position_factor});
    row["alignment"] = json!(match v.alignment {
        TextTabAlignment::Left => "left",
        TextTabAlignment::Center => "center",
        TextTabAlignment::Right => "right",
        TextTabAlignment::Decimal { separator } => {
            row["separator"] = json!(separator);
            "decimal"
        }
    });
    row
}
fn read_tab(v: &Value) -> Option<TextTabStop> {
    Some(TextTabStop {
        position_factor: v.get("positionFactor")?.as_f64()?,
        alignment: match v.get("alignment")?.as_str()? {
            "left" => TextTabAlignment::Left,
            "center" => TextTabAlignment::Center,
            "right" => TextTabAlignment::Right,
            "decimal" => TextTabAlignment::Decimal {
                separator: scalar_char(v.get("separator")?)?,
            },
            _ => return None,
        },
    })
}
pub(super) fn paragraph(v: &ParagraphFormat) -> Value {
    let mut row = json!({});
    macro_rules! fields { ($($field:ident=>$key:literal),*) => { $(if let Some(v)=v.$field { row[$key]=json!(v); })* }; }
    fields!(left_indent_factor=>"leftIndentFactor",right_indent_factor=>"rightIndentFactor",first_line_indent_factor=>"firstLineIndentFactor",space_before=>"spaceBefore",space_after=>"spaceAfter");
    if let Some(v) = v.alignment {
        row["alignment"] = paragraph_alignment(v);
    }
    if let Some(v) = v.line_spacing {
        row["lineSpacing"] = spacing(v);
    }
    if let Some(v) = &v.tab_stops {
        row["tabStops"] = Value::Array(v.iter().map(tab).collect());
    }
    row
}
pub(super) fn read_paragraph(v: &Value) -> Option<ParagraphFormat> {
    Some(ParagraphFormat {
        alignment: optional(v.get("alignment"), read_paragraph_alignment)?,
        line_spacing: optional(v.get("lineSpacing"), read_spacing)?,
        tab_stops: optional(v.get("tabStops"), |v| {
            v.as_array()?.iter().map(read_tab).collect()
        })?,
        left_indent_factor: optional(v.get("leftIndentFactor"), Value::as_f64)?,
        right_indent_factor: optional(v.get("rightIndentFactor"), Value::as_f64)?,
        first_line_indent_factor: optional(v.get("firstLineIndentFactor"), Value::as_f64)?,
        space_before: optional(v.get("spaceBefore"), Value::as_f64)?,
        space_after: optional(v.get("spaceAfter"), Value::as_f64)?,
    })
}
fn inline(v: &MTextInline<IfccadColor>) -> Value {
    match v {
        MTextInline::Run {
            text,
            character_format,
        } => json!({"kind":"run","text":text,"characterFormat":character(character_format)}),
        MTextInline::Tab => json!({"kind":"tab"}),
        MTextInline::LineBreak => json!({"kind":"lineBreak"}),
        MTextInline::ColumnBreak => json!({"kind":"columnBreak"}),
        MTextInline::Stack(v) => {
            let mut row = json!({"kind":"stack","stackKind":stack_kind(v.stack_kind),"upper":v.upper,"lower":v.lower,"alignment":stack_alignment(v.alignment),"textScale":v.text_scale,"characterFormat":character(&v.character_format)});
            if let Some(v) = v.separator {
                row["separator"] = json!(v);
            }
            row
        }
    }
}
fn read_inline(v: &Value) -> Option<MTextInline<IfccadColor>> {
    Some(match v.get("kind")?.as_str()? {
        "run" => MTextInline::Run {
            text: v.get("text")?.as_str()?.into(),
            character_format: optional(v.get("characterFormat"), read_character)?
                .unwrap_or_default(),
        },
        "tab" => MTextInline::Tab,
        "lineBreak" => MTextInline::LineBreak,
        "columnBreak" => MTextInline::ColumnBreak,
        "stack" => MTextInline::Stack(TextStack {
            stack_kind: read_stack_kind(v.get("stackKind")?)?,
            upper: v.get("upper")?.as_str()?.into(),
            lower: v.get("lower")?.as_str()?.into(),
            alignment: optional(v.get("alignment"), read_stack_alignment)?.unwrap_or_default(),
            text_scale: optional(v.get("textScale"), Value::as_f64)?.unwrap_or(0.7),
            separator: optional(v.get("separator"), scalar_char)?,
            character_format: optional(v.get("characterFormat"), read_character)?
                .unwrap_or_default(),
        }),
        _ => return None,
    })
}
pub(super) fn content(v: &[MTextParagraph<IfccadColor>]) -> Value {
    Value::Array(v.iter().map(|p| json!({"paragraphFormat":paragraph(&p.paragraph_format),"characterFormat":character(&p.character_format),"inlines":p.inlines.iter().map(inline).collect::<Vec<_>>()})).collect())
}
pub(super) fn read_content(v: &Value) -> Option<Vec<MTextParagraph<IfccadColor>>> {
    v.as_array()?
        .iter()
        .map(|p| {
            Some(MTextParagraph {
                paragraph_format: optional(p.get("paragraphFormat"), read_paragraph)?
                    .unwrap_or_default(),
                character_format: optional(p.get("characterFormat"), read_character)?
                    .unwrap_or_default(),
                inlines: p
                    .get("inlines")?
                    .as_array()?
                    .iter()
                    .map(read_inline)
                    .collect::<Option<Vec<_>>>()?,
            })
        })
        .collect()
}
pub(super) fn columns(v: &MTextColumns) -> Value {
    match v {
        MTextColumns::Static {
            count,
            column_width,
            gutter,
            column_height,
            flow_reversed,
        } => {
            json!({"kind":"static","count":count,"columnWidth":column_width,"gutter":gutter,"columnHeight":column_height,"flowReversed":flow_reversed})
        }
        MTextColumns::DynamicAutoHeight {
            column_width,
            gutter,
            column_height,
            current_column_count,
            flow_reversed,
        } => {
            json!({"kind":"dynamicAutoHeight","columnWidth":column_width,"gutter":gutter,"columnHeight":column_height,"currentColumnCount":current_column_count,"flowReversed":flow_reversed})
        }
        MTextColumns::DynamicManualHeight {
            column_width,
            gutter,
            column_heights,
            flow_reversed,
        } => {
            json!({"kind":"dynamicManualHeight","columnWidth":column_width,"gutter":gutter,"columnHeights":column_heights.iter().map(|h| match h {MTextColumnHeight::Fixed{distance}=>json!({"kind":"fixed","distance":distance}),MTextColumnHeight::Auto=>json!({"kind":"auto"})}).collect::<Vec<_>>(),"flowReversed":flow_reversed})
        }
    }
}
pub(super) fn read_columns(v: &Value) -> Option<MTextColumns> {
    let column_width = v.get("columnWidth")?.as_f64()?;
    let gutter = v.get("gutter")?.as_f64()?;
    let flow_reversed = optional(v.get("flowReversed"), Value::as_bool)?.unwrap_or(false);
    Some(match v.get("kind")?.as_str()? {
        "static" => MTextColumns::Static {
            count: u32::try_from(v.get("count")?.as_u64()?).ok()?,
            column_width,
            gutter,
            column_height: v.get("columnHeight")?.as_f64()?,
            flow_reversed,
        },
        "dynamicAutoHeight" => MTextColumns::DynamicAutoHeight {
            current_column_count: u32::try_from(v.get("currentColumnCount")?.as_u64()?).ok()?,
            column_width,
            gutter,
            column_height: v.get("columnHeight")?.as_f64()?,
            flow_reversed,
        },
        "dynamicManualHeight" => MTextColumns::DynamicManualHeight {
            column_width,
            gutter,
            column_heights: v
                .get("columnHeights")?
                .as_array()?
                .iter()
                .map(|h| {
                    Some(match h.get("kind")?.as_str()? {
                        "fixed" => MTextColumnHeight::Fixed {
                            distance: h.get("distance")?.as_f64()?,
                        },
                        "auto" => MTextColumnHeight::Auto,
                        _ => return None,
                    })
                })
                .collect::<Option<Vec<_>>>()?,
            flow_reversed,
        },
        _ => return None,
    })
}
pub(super) fn background(v: &MTextBackground<IfccadColor>) -> Value {
    json!({"fill":match &v.fill {MTextFill::None=>json!({"kind":"none"}),MTextFill::Canvas=>json!({"kind":"canvas"}),MTextFill::Color(c)=>json!({"kind":"color","color":json!(c)})},
        "padding":match v.padding {TextPadding::Absolute{distance}=>json!({"kind":"absolute","distance":distance}),TextPadding::Relative{factor}=>json!({"kind":"relative","factor":factor})},"opacity":v.opacity,"frame":v.frame})
}
pub(super) fn read_background(v: &Value) -> Option<MTextBackground<IfccadColor>> {
    let defaults = MTextBackground::default();
    Some(MTextBackground {
        fill: optional(v.get("fill"), |v| {
            Some(match v.get("kind")?.as_str()? {
                "none" => MTextFill::None,
                "canvas" => MTextFill::Canvas,
                "color" => MTextFill::Color(super::presentation::read_color(v.get("color")?)?),
                _ => return None,
            })
        })?
        .unwrap_or(defaults.fill),
        padding: optional(v.get("padding"), |v| {
            Some(match v.get("kind")?.as_str()? {
                "absolute" => TextPadding::Absolute {
                    distance: v.get("distance")?.as_f64()?,
                },
                "relative" => TextPadding::Relative {
                    factor: v.get("factor")?.as_f64()?,
                },
                _ => return None,
            })
        })?
        .unwrap_or(defaults.padding),
        opacity: optional(v.get("opacity"), Value::as_f64)?.unwrap_or(defaults.opacity),
        frame: optional(v.get("frame"), Value::as_bool)?.unwrap_or(defaults.frame),
    })
}
