//! IFCCAD drawing model, shared validation and IFCX JSON / experimental CBOR IO.
//!
//! Public model and IO types are exported directly from this crate. Geometry,
//! text, dimensions, line patterns, plot and workspace kernels remain independent
//! of CAD runtimes. Active CAD exchange uses the shared `opencaddocument` route in the viewer.
#![allow(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
pub mod color;
pub mod dimension;
pub mod geometry_kernel;
pub(crate) mod ifccad;
pub use ifccad::*;
pub mod line_pattern;

pub mod plot_kernel;
pub mod text;

pub mod workspace_kernel;

pub(crate) mod names;
