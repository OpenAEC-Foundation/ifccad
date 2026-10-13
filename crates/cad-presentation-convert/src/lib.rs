//! CAD presentation scalars without native drawing identities or diagnostics.

use opencadcodec::{Color, LineWeight, Transparency};

mod color;
mod line_pattern;
pub use line_pattern::*;
mod quantization;
mod viewport_overrides;
pub use viewport_overrides::*;
mod xrecord_sections;
pub use color::{color_to_cad, explicit_color_from_cad};
pub use quantization::{lineweight_to_cad, opacity_to_cad};
pub use xrecord_sections::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CadColorValue {
    Foreground,
    Concrete {
        rgb: [u8; 3],
        indexed: Option<(String, u64)>,
        named: Option<(String, String)>,
    },
}

impl CadColorValue {
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
    pub fn concrete_rgb(&self) -> Option<[u8; 3]> {
        match self {
            Self::Concrete { rgb, .. } => Some(*rgb),
            Self::Foreground => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CadColorLoss {
    UnsupportedIndex,
    InconsistentIndex,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CadColorMapping {
    pub color: Color,
    pub named: Option<(String, String)>,
    pub losses: Vec<CadColorLoss>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CadOpacityMapping {
    pub transparency: Transparency,
    pub roundtrip: f64,
    pub changed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CadLineweightMapping {
    pub weight: LineWeight,
    pub roundtrip_mm: f64,
    pub changed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PresentationValueError {
    #[error("opacity must be finite and in [0, 1]")]
    InvalidOpacity,
    #[error("lineweight must be finite and nonnegative")]
    InvalidLineweight,
    #[error("concrete color metadata must have nonempty identity strings")]
    InvalidColorIdentity,
    #[error("color has no supported concrete RGB/ACI meaning")]
    UnsupportedColor,
    #[error("viewport override has an unsupported type, method or value")]
    UnsupportedOverride,
}
