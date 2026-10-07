//! Experimental IFCCAD drawing profile using IFCX, independent of standalone OCDraw.
//!
//! The formats share geometric validation types and the OCDraw unit registry.
//! IFCX composition, nodes, schema imports and CAD attributes stay in this module.
//!
//! The owned [`IfccadDocument`] is a CAD projection; [`LoadedIfccadGraph`] retains
//! its complete immutable source. Encoding produces a fresh CAD-profile file.
//!
//! ```
//! use ocdraw::ifccad::{load_ifccad_bytes, validate_ifccad_document,
//!     encode_ifccad_document};
//! # fn edit(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
//! let (source, mut document) = load_ifccad_bytes(bytes, Default::default())?.into_parts();
//! document.model.entities.reverse();
//! validate_ifccad_document(&document)?;
//! let profile_bytes = encode_ifccad_document(&document)?;
//! // Original download retains all source information; profile encoding is separate.
//! let original_bytes = source.source_bytes();
//! # let _ = (profile_bytes, original_bytes);
//! # Ok(())
//! # }
//! ```
//!
//! Add an empty, unsized sheet without inventing a physical paper format:
//! ```
//! use ocdraw::ifccad::{load_ifccad_bytes, encode_ifccad_document,
//!     IfccadPaperLayout};
//! # fn add_sheet(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
//! let mut document = load_ifccad_bytes(bytes, Default::default())?.into_document();
//! let id = document.id_counters.allocate_layout_id()?;
//! let tab_index = u32::try_from(document.paper_layouts.len() + 1)?;
//! document.paper_layouts.push(IfccadPaperLayout { bounds: None,
//!     id, name: "New sheet".into(), tab_index,
//!     settings: Default::default(), entities: vec![],
//! });
//! let encoded = encode_ifccad_document(&document)?;
//! # let _ = encoded;
//! # Ok(())
//! # }
//! ```

mod codec;
mod encode;
mod logical;
mod read;
mod source;
mod storage;
pub(crate) const PROFILE_URI: &str = "urn:example:ifccad:0.1.0";
pub use encode::{encode_ifccad_document, IfccadEncodeError};
pub use logical::*;
pub use read::{load_ifccad_bytes, IfccadReadError, IfccadReadOptions, ValidatedIfccad};
pub use source::LoadedIfccadGraph;
pub use storage::{load_ifccad_file, EncodedIfccad, IfccadOpenError, IfccadWriteError};
/// How repeated IFCX node paths are composed before CAD validation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IfccadCompositionPolicy {
    /// Merge keyed node fields in file order; later values win.
    #[default]
    LaterWins,
    /// Reject a repeated field or key when its value differs.
    RejectConflicts,
}

/// Errors found while loading or writing the experimental profile.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("IFCCAD validation failed: {errors:?}")]
pub struct IfccadReport {
    /// Human-readable, location-oriented diagnostics.
    pub errors: Vec<String>,
}

impl IfccadReport {
    pub(crate) fn one(message: impl Into<String>) -> Self {
        Self {
            errors: vec![message.into()],
        }
    }
}

// Independent public paths for shared ID-free layout output values.
pub use crate::plot_kernel::{
    LayoutMedia as IfccadLayoutMedia, LayoutOutputSettings as IfccadLayoutSettings,
    MediaUnit as IfccadMediaUnit, PlotSettings as IfccadPlotSettings,
    PlotStyleMode as IfccadPlotStyleMode,
};
