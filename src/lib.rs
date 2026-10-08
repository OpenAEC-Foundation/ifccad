//! Open CAD Drawing (OCDraw): an open, application-independent information
//! model and exchange format for CAD drawings.
//!
//! The typed logical model and shared validation are separate from the JSON
//! mapping. CAD conversion lives in the companion `ocdraw-convert` crate.
//! The independent experimental IFCCAD drawing profile lives in [`ifccad`],
//! with direct CAD conversion in the companion `ifccad-convert` crate.
#![allow(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
pub mod geometry_kernel;
pub mod ifccad;
pub mod ocdraw;

pub mod plot_kernel;
pub mod text;

pub mod workspace_kernel;
