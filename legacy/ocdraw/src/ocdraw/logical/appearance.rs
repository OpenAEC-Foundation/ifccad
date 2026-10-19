//! Native fixed and background-dependent colors.

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DrawingColor {
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
pub enum DrawingColorMetadataError {
    #[error("foreground color cannot carry indexed or named metadata")]
    ForegroundMetadata,
}

impl DrawingColor {
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
    ) -> Result<Self, DrawingColorMetadataError> {
        let Self::Concrete { indexed, .. } = &mut self else {
            return Err(DrawingColorMetadataError::ForegroundMetadata);
        };
        *indexed = Some((system.into(), index));
        Ok(self)
    }

    pub fn with_named(
        mut self,
        catalog: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Self, DrawingColorMetadataError> {
        let Self::Concrete { named, .. } = &mut self else {
            return Err(DrawingColorMetadataError::ForegroundMetadata);
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
