//! Authored values. No identities, placements, CAD types or physical encoding.

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FontRequest {
    pub family: Option<String>,
    pub cad_font_name: Option<String>,
    pub big_font_name: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub charset: Option<u16>,
    pub pitch: Option<u8>,
}

impl FontRequest {
    pub fn family(name: impl Into<String>) -> Self {
        Self {
            family: Some(name.into()),
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextStyleProperties {
    pub font: FontRequest,
    pub width_factor: f64,
    pub oblique_angle: f64,
    pub vertical: bool,
    pub creation_height: Option<f64>,
    pub last_used_height: Option<f64>,
    pub creation_backward: bool,
    pub creation_upside_down: bool,
}

impl TextStyleProperties {
    pub fn new(font: FontRequest) -> Self {
        Self {
            font,
            width_factor: 1.0,
            oblique_angle: 0.0,
            vertical: false,
            creation_height: None,
            last_used_height: None,
            creation_backward: false,
            creation_upside_down: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextHeight {
    Relative { factor: f64 },
    Absolute { distance: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextLineSpacing {
    Exact { distance: f64 },
    AtLeast { distance: f64 },
    Multiple { factor: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum TextColor<C> {
    Entity,
    ByLayer,
    ByBlock,
    Explicit(C),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextPosition {
    #[default]
    Bottom,
    Center,
    Top,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextRun {
    pub text: String,
    pub underline: bool,
    pub overline: bool,
    pub strike_through: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextHorizontalAlignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextVerticalAlignment {
    Baseline,
    Bottom,
    Middle,
    Top,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextLayout {
    Anchored {
        horizontal: TextHorizontalAlignment,
        vertical: TextVerticalAlignment,
        height: f64,
        width_factor: f64,
    },
    WholeTextMiddle {
        height: f64,
        width_factor: f64,
    },
    Aligned {
        length: f64,
        width_factor: f64,
    },
    Fit {
        length: f64,
        height: f64,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CharacterFormat<C> {
    pub font: Option<FontRequest>,
    pub color: Option<TextColor<C>>,
    pub height: Option<TextHeight>,
    pub width_factor: Option<f64>,
    pub tracking: Option<f64>,
    pub oblique_angle: Option<f64>,
    pub underline: Option<bool>,
    pub overline: Option<bool>,
    pub strike_through: Option<bool>,
    pub position: Option<TextPosition>,
}

impl<C> Default for CharacterFormat<C> {
    fn default() -> Self {
        Self {
            font: None,
            color: None,
            height: None,
            width_factor: None,
            tracking: None,
            oblique_angle: None,
            underline: None,
            overline: None,
            strike_through: None,
            position: None,
        }
    }
}

/// Derived values are never written over their authored overrides.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedCharacterFormat<C> {
    pub font: FontRequest,
    pub color: TextColor<C>,
    pub height: f64,
    pub width_factor: f64,
    pub tracking: f64,
    pub oblique_angle: f64,
    pub underline: bool,
    pub overline: bool,
    pub strike_through: bool,
    pub position: TextPosition,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextParagraphAlignment {
    #[default]
    Left,
    Center,
    Right,
    Justified,
    Distributed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextTabAlignment {
    Left,
    Center,
    Right,
    Decimal { separator: char },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextTabStop {
    pub position_factor: f64,
    pub alignment: TextTabAlignment,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParagraphFormat {
    pub alignment: Option<TextParagraphAlignment>,
    pub left_indent_factor: Option<f64>,
    pub right_indent_factor: Option<f64>,
    pub first_line_indent_factor: Option<f64>,
    pub space_before: Option<f64>,
    pub space_after: Option<f64>,
    pub line_spacing: Option<TextLineSpacing>,
    /// None inherits; Some(empty) explicitly resets to the standard grid.
    pub tab_stops: Option<Vec<TextTabStop>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedParagraphFormat {
    pub alignment: TextParagraphAlignment,
    pub left_indent_factor: f64,
    pub right_indent_factor: f64,
    pub first_line_indent_factor: f64,
    pub space_before: f64,
    pub space_after: f64,
    pub line_spacing: TextLineSpacing,
    pub tab_stops: Vec<TextTabStop>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MTextParagraph<C> {
    pub paragraph_format: ParagraphFormat,
    pub character_format: CharacterFormat<C>,
    pub inlines: Vec<MTextInline<C>>,
}

impl<C> Default for MTextParagraph<C> {
    fn default() -> Self {
        Self {
            paragraph_format: Default::default(),
            character_format: Default::default(),
            inlines: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MTextInline<C> {
    Run {
        text: String,
        character_format: CharacterFormat<C>,
    },
    Tab,
    LineBreak,
    ColumnBreak,
    Stack(TextStack<C>),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextStackKind {
    #[default]
    Fraction,
    DiagonalFraction,
    Tolerance,
    DecimalTolerance,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextStackAlignment {
    Top,
    #[default]
    Center,
    Bottom,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextStack<C> {
    pub stack_kind: TextStackKind,
    pub upper: String,
    pub lower: String,
    pub alignment: TextStackAlignment,
    pub text_scale: f64,
    pub separator: Option<char>,
    pub character_format: CharacterFormat<C>,
}

impl<C> Default for TextStack<C> {
    fn default() -> Self {
        Self {
            stack_kind: Default::default(),
            upper: String::new(),
            lower: String::new(),
            alignment: Default::default(),
            text_scale: 0.7,
            separator: None,
            character_format: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MTextFlow {
    #[default]
    Horizontal,
    Vertical,
    ByStyle,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MTextAttachment {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MTextColumnHeight {
    Fixed { distance: f64 },
    Auto,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MTextColumns {
    Static {
        count: u32,
        column_width: f64,
        gutter: f64,
        column_height: f64,
        flow_reversed: bool,
    },
    DynamicAutoHeight {
        column_width: f64,
        gutter: f64,
        column_height: f64,
        current_column_count: u32,
        flow_reversed: bool,
    },
    DynamicManualHeight {
        column_width: f64,
        gutter: f64,
        column_heights: Vec<MTextColumnHeight>,
        flow_reversed: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MTextValueContext<'a> {
    pub nominal_height: f64,
    pub wrap_width: Option<f64>,
    pub columns: Option<&'a MTextColumns>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextPadding {
    Absolute { distance: f64 },
    Relative { factor: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum MTextFill<C> {
    None,
    Color(C),
    Canvas,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MTextBackground<C> {
    pub fill: MTextFill<C>,
    pub padding: TextPadding,
    pub opacity: f64,
    pub frame: bool,
}

impl<C> Default for MTextBackground<C> {
    fn default() -> Self {
        Self {
            fill: MTextFill::None,
            padding: TextPadding::Relative { factor: 0.5 },
            opacity: 1.0,
            frame: false,
        }
    }
}
