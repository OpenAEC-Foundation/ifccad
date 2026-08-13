//! Directory-based IFCCAD package foundations.
//!
//! The directory loader remains an internal implementation detail. This module
//! publicly exposes only the stable diagnostic vocabulary.

mod analysis;
pub(crate) mod codes;
mod diagnostic;
mod discovery;
mod error;
mod graph;
mod loader;
mod model;
mod path;
mod schema;
mod uri;
mod validation;

pub use diagnostic::{
    PackageDiagnostic, PackageDiagnosticContextValue, PackageDiagnosticSeverity,
    PackageValidationReport,
};
pub(crate) use error::PackageOpenError;
pub(crate) use model::LoadedJsonResource;
#[cfg(test)]
pub(crate) use validation::load_directory_package;

/// Current IFCX entrypoint inside an exploded directory package.
pub(crate) const DIRECTORY_PACKAGE_ENTRYPOINT: &str = "package.ifcx.json";
