//! CAD-only preparation of ordinary dimensions; native identities remain external.
mod association;
mod diagnostics;
mod overrides;
mod picture;
mod placement;
pub use picture::*;
mod source_overrides;
mod styles;
pub use association::*;
pub use cad_presentation_convert::CadColorValue;
pub use diagnostics::*;
pub use overrides::*;
pub use placement::*;
pub use source_overrides::*;
pub use styles::*;
pub type CadDimensionStyleValues = ifccad::dimension::DimensionStyleSettings<
    opencadcodec::Handle,
    opencadcodec::Handle,
    CadColorValue,
    opencadcodec::Handle,
>;
pub type CadDimensionStyleOverrides = ifccad::dimension::DimensionStyleOverrides<
    opencadcodec::Handle,
    opencadcodec::Handle,
    CadColorValue,
    opencadcodec::Handle,
>;
