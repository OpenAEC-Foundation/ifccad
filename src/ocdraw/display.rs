//! Drawing-local point display setting.

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
    pub(crate) fn is_valid(self) -> bool {
        match self.size {
            PointSize::DefaultFivePercent => true,
            PointSize::Absolute(value) | PointSize::ViewportPercent(value) => {
                value.is_finite() && value > 0.0
            }
        }
    }
}
