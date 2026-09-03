//! Directory-based IFCCAD package foundations.
//!
//! Packages can be inspected for diagnostics and exposed through a strict,
//! typed model only after all required validation succeeds.

mod analysis;
mod appearance;
mod bindings;
pub(crate) mod codes;
mod diagnostic;
mod discovery;
mod error;
mod graph;
mod header;
mod loader;
mod model;
mod navigation;
mod path;
mod schema;
mod uri;
mod validation;

pub use analysis::ValidatedIfccadPackage;
pub use appearance::{
    AppearanceColorRef, AppearanceProperty, IndexedColorRef, LinePatternRef, NamedColorRef,
    RgbColor,
};
pub use diagnostic::{
    PackageDiagnostic, PackageDiagnosticContextValue, PackageDiagnosticSeverity,
    PackageValidationReport,
};
pub use error::PackageOpenError;
pub(crate) use model::LoadedJsonResource;
pub use model::PackageLoadOutcome;
pub use navigation::{
    AppearanceRef, AppliedAppearanceRef, DrawingLayoutKind, DrawingLayoutRef, DrawingRef,
    DrawingSetRef, GeometryRepresentationRef, LayerRef,
};
pub use validation::load_directory_package;

/// Current IFCX entrypoint inside an exploded directory package.
pub const DIRECTORY_PACKAGE_ENTRYPOINT: &str = "package.ifcx.json";
