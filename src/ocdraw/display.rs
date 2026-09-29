//! Drawing-local point display setting.

use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointGlyph {
    Dot,
    Hidden,
    Plus,
    Cross,
    ShortLine,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointSize {
    DefaultFivePercent,
    Absolute(f64),
    ViewportPercent(f64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointDisplay {
    pub glyph: PointGlyph,
    pub circle: bool,
    pub square: bool,
    pub size: PointSize,
}

impl Default for PointDisplay {
    fn default() -> Self {
        Self {
            glyph: PointGlyph::Dot,
            circle: false,
            square: false,
            size: PointSize::DefaultFivePercent,
        }
    }
}

impl PointDisplay {
    pub(crate) fn to_json(self) -> Option<Value> {
        let glyph = match self.glyph {
            PointGlyph::Dot => "dot",
            PointGlyph::Hidden => "hidden",
            PointGlyph::Plus => "plus",
            PointGlyph::Cross => "cross",
            PointGlyph::ShortLine => "shortLine",
        };
        let size = match self.size {
            PointSize::DefaultFivePercent => json!({"kind":"defaultFivePercent"}),
            PointSize::Absolute(value) if value.is_finite() && value > 0.0 => {
                json!({"kind":"absolute","value":value})
            }
            PointSize::ViewportPercent(value) if value.is_finite() && value > 0.0 => {
                json!({"kind":"viewportPercent","value":value})
            }
            _ => return None,
        };
        Some(json!({"form":{"glyph":glyph,"circle":self.circle,"square":self.square},"size":size}))
    }
}
