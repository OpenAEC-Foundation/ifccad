//! Entity appearance choices and color meaning, independent of JSON columns.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrawingColor {
    pub rgb: [u8; 3],
    pub indexed: Option<(String, u64)>,
    pub named: Option<(String, String)>,
}

impl DrawingColor {
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
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppearanceSelection<T> {
    ByLayer,
    ByBlock,
    Explicit(T),
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntityAppearance {
    pub color: AppearanceSelection<DrawingColor>,
    pub opacity: AppearanceSelection<f64>,
    pub line_pattern: AppearanceSelection<super::LinePatternId>,
    pub line_pattern_scale: f64,
    pub line_weight: AppearanceSelection<f64>,
}

impl Default for EntityAppearance {
    fn default() -> Self {
        Self {
            color: AppearanceSelection::ByLayer,
            opacity: AppearanceSelection::ByLayer,
            line_pattern: AppearanceSelection::ByLayer,
            line_pattern_scale: 1.0,
            line_weight: AppearanceSelection::ByLayer,
        }
    }
}
