//! CAD-only preparation of ordinary dimensions; native identities remain external.
mod association;
mod diagnostics;
mod overrides;
mod placement;
mod styles;
pub use association::*;
pub use cad_presentation_convert::CadColorValue;
pub use diagnostics::*;
pub use overrides::*;
pub use placement::*;
pub use styles::*;
pub type CadDimensionStyleValues = ocdraw::dimension::DimensionStyleSettings<
    opencadcodec::Handle,
    opencadcodec::Handle,
    CadColorValue,
    opencadcodec::Handle,
>;
pub type CadDimensionStyleOverrides = ocdraw::dimension::DimensionStyleOverrides<
    opencadcodec::Handle,
    opencadcodec::Handle,
    CadColorValue,
    opencadcodec::Handle,
>;
