//! Native fixed and background-dependent colors.

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IfccadColor {
    /// Black or white chosen against the occurrence's effective background.
    Foreground,
    /// Fixed RGB with optional palette/catalog identity.
    Concrete {
        rgb: [u8; 3],
        indexed: Option<(String, u64)>,
        named: Option<(String, String)>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum IfccadColorMetadataError {
    #[error("foreground color cannot carry indexed or named metadata")]
    ForegroundMetadata,
}

impl IfccadColor {
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::Concrete {
            rgb: [red, green, blue],
            indexed: None,
            named: None,
        }
    }

    pub fn with_indexed(
        mut self,
        system: impl Into<String>,
        index: u64,
    ) -> Result<Self, IfccadColorMetadataError> {
        let Self::Concrete { indexed, .. } = &mut self else {
            return Err(IfccadColorMetadataError::ForegroundMetadata);
        };
        *indexed = Some((system.into(), index));
        Ok(self)
    }

    pub fn with_named(
        mut self,
        catalog: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Self, IfccadColorMetadataError> {
        let Self::Concrete { named, .. } = &mut self else {
            return Err(IfccadColorMetadataError::ForegroundMetadata);
        };
        *named = Some((catalog.into(), name.into()));
        Ok(self)
    }

    pub fn concrete_rgb(&self) -> Option<[u8; 3]> {
        match self {
            Self::Concrete { rgb, .. } => Some(*rgb),
            Self::Foreground => None,
        }
    }

    pub fn indexed_identity(&self) -> Option<&(String, u64)> {
        match self {
            Self::Concrete { indexed, .. } => indexed.as_ref(),
            Self::Foreground => None,
        }
    }

    pub fn named_identity(&self) -> Option<&(String, String)> {
        match self {
            Self::Concrete { named, .. } => named.as_ref(),
            Self::Foreground => None,
        }
    }

    pub(crate) fn is_valid(&self) -> bool {
        let Self::Concrete { indexed, named, .. } = self else {
            return true;
        };
        indexed.as_ref().is_none_or(|(system, index)| {
            !system.is_empty() && !(system.eq_ignore_ascii_case("ACI") && *index == 7)
        }) && named
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
