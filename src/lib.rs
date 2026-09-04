//! Rust foundations for the IFCCAD exchange format.
//!
//! IFCCAD combines an IFCX semantic graph with one or more IFCDR drawing
//! resources and optional IFCPR preservation resources. A package may be
//! CAD-only or include a broader project, building, and product graph.
//!
//! The public API is organised around package inspection and package creation,
//! with shared IFCDR drawing-resource types exposed separately. Read and write
//! implementation modules remain private so those responsibilities can evolve
//! without becoming part of the public module structure.

#![allow(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod canonicalization;
pub mod conformance;
mod diagnostic;
mod json_resource;
pub mod package;
mod package_id;
mod resource;
pub use package_id::{InvalidPackageId, PackageId};
pub use resource::{InvalidResourceId, ResourceId};

pub mod ifcdr;
mod validated;
