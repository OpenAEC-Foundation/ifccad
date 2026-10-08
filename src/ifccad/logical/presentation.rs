//! IFCCAD-owned concrete presentation values, independent of CAD runtimes.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfccadColor {
    pub rgb: [u8; 3],
    pub indexed: Option<(String, u64)>,
    pub named: Option<(String, String)>,
}

impl IfccadColor {
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            rgb: [red, green, blue],
            indexed: None,
            named: None,
        }
    }
    pub fn with_indexed(mut self, system: impl Into<String>, index: u64) -> Self {
        self.indexed = Some((system.into(), index));
        self
    }
    pub fn with_named(mut self, catalog: impl Into<String>, name: impl Into<String>) -> Self {
        self.named = Some((catalog.into(), name.into()));
        self
    }
    pub(crate) fn is_valid(&self) -> bool {
        self.indexed
            .as_ref()
            .is_none_or(|(system, _)| !system.is_empty())
            && self
                .named
                .as_ref()
                .is_none_or(|(catalog, name)| !catalog.is_empty() && !name.is_empty())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadPointGlyph {
    Dot,
    Hidden,
    Plus,
    Cross,
    ShortLine,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IfccadPointSize {
    DefaultFivePercent,
    Absolute(f64),
    ViewportPercent(f64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IfccadPointDisplay {
    pub glyph: IfccadPointGlyph,
    pub circle: bool,
    pub square: bool,
    pub size: IfccadPointSize,
}
impl Default for IfccadPointDisplay {
    fn default() -> Self {
        Self {
            glyph: IfccadPointGlyph::Dot,
            circle: false,
            square: false,
            size: IfccadPointSize::DefaultFivePercent,
        }
    }
}
impl IfccadPointDisplay {
    pub(crate) fn is_valid(self) -> bool {
        match self.size {
            IfccadPointSize::DefaultFivePercent => true,
            IfccadPointSize::Absolute(v) | IfccadPointSize::ViewportPercent(v) => {
                v.is_finite() && v > 0.0
            }
        }
    }
}
