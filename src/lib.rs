//! Rust foundations for the IFCCAD exchange format.
//!
//! IFCCAD combines an IFCX semantic graph with one or more IFCDR drawing
//! resources and optional IFCPR preservation resources. A package may be
//! CAD-only or include a broader project, building, and product graph.
//!
//! This crate currently exposes canonicalisation, fingerprints, conformance
//! support, and the stable package diagnostic vocabulary. Package loading and
//! the wider format and conversion model remain under development.

#![allow(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod canonicalization;
pub mod conformance;
pub mod package;

mod validated;
