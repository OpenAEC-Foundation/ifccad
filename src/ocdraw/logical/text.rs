//! Drawing-owned text identity and authored values. Font resolution is external.
use super::{DrawingColor, EntityAppearance};
use crate::{geometry_kernel::CoordinateFrame3, text::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OcdrawTextStyleId(pub u32);

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingTextStyle {
    pub id: OcdrawTextStyleId,
    pub name: String,
    pub properties: TextStyleProperties,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingTextEntity {
    pub id: u64,
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

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingMTextEntity {
    pub id: u64,
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
