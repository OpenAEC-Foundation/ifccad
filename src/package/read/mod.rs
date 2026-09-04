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

pub(crate) use crate::diagnostic::{
    PackageDiagnostic, PackageDiagnosticContextValue, PackageDiagnosticSeverity,
};
pub use analysis::ValidatedPackage;
pub use appearance::{
    AppearanceColorRef, AppearanceProperty, IndexedColorRef, LinePatternRef, NamedColorRef,
    RgbColor,
};
pub use diagnostic::PackageValidationReport;
pub use error::PackageOpenError;
pub(crate) use header::canonical_rfc3339_utc;
pub use header::PackageHeaderRef;
pub use model::PackageLoadOutcome;
pub use navigation::{
    AppearanceRef, AppliedAppearanceRef, DrawingLayoutKind, DrawingLayoutRef, DrawingRef,
    DrawingSetRef, GeometryRepresentationRef, LayerRef,
};
pub use validation::load_directory_package;

pub(super) use super::DIRECTORY_PACKAGE_ENTRYPOINT;
