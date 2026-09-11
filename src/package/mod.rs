//! Directory-based IFCCAD package foundations.
//!
//! Packages can be inspected for diagnostics and exposed through a strict,
//! typed model only after all required validation succeeds.
//!
//! This module is the public facade for both directions: loading an existing
//! package and building a new one. The underlying `read` and `write` modules
//! are deliberately private implementation details.

mod read;
mod write;

pub use crate::diagnostic::{
    PackageDiagnostic, PackageDiagnosticCategory, PackageDiagnosticContextValue,
    PackageDiagnosticSeverity,
};
pub(crate) use read::canonical_rfc3339_utc;
pub use read::{
    load_directory_package, AppearanceColorRef, AppearanceProperty, AppearanceRef,
    AppliedAppearanceRef, AssessmentCompleteness, AssessmentGap, AssessmentGapReason,
    DrawingLayoutKind, DrawingLayoutRef, DrawingRef, DrawingRepresentationRef, DrawingSetRef,
    IndexedColorRef, LayerRef, LinePatternRef, NamedColorRef, PackageAssessment, PackageHeaderRef,
    PackageLoadOutcome, PackageOpenError, PackageValidationReport, PackageValidity, RgbColor,
    ValidatedPackage,
};
pub use write::{
    AppearanceColor, AppearanceDefinition, AppearanceKey, AppearanceMode, DrawingAppearances,
    DrawingBuilder, DrawingLayers, DrawingOptions, DrawingResourceStorage, EncodedPackage,
    EntityAppearance, LayerDefinition, LayerKey, LineDefinition, LinePatternDefinition,
    ModelSpaceBuilder, PackageBuildError, PackageBuilder, PackageOptions, PackageWriteError,
    PolylineDefinition,
};

/// Current IFCX entrypoint inside an exploded directory package.
pub const DIRECTORY_PACKAGE_ENTRYPOINT: &str = "package.ifcx.json";
