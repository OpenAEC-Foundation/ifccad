mod access;
mod block_bounds;
mod blocks;
pub(crate) use blocks::BlockGraph;
mod diagnostic;
mod types;
mod validation;
mod viewport;

pub(crate) use access::*;
pub(crate) use diagnostic::*;
pub(crate) use types::*;
pub use types::{AppearanceMode, IfcdrColor, IfcdrIndexedColor, IfcdrNamedColor};
pub use types::{
    BackClip, BackClipMode, FrontClip, FrontClipMode, PaperClip, ProjectionMode, ShadedPlot,
    ShadedPlotMode, ShadedPlotQuality, ShadedPlotQualityMode, ViewDefinition, ViewportFrame,
    ViewportLayerOverride, ViewportRenderMode,
};
pub(crate) use validation::*;

#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;
