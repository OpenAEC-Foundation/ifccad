//! Open CAD Drawing (OCDraw): an open, application-independent information
//! model and exchange format for CAD drawings.
//!
//! The typed logical model and shared validation are separate from the JSON
//! mapping. CAD conversion lives in the companion `ocdraw-convert` crate.
#![allow(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
pub mod experimental_ifcx;
pub mod ocdraw;
