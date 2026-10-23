//! ID-free planar Hatch parameters, join proofs and conservative enclosures.
mod bounds;
mod editing;
mod endpoints;
pub use editing::*;
mod gradient;
mod pattern;
mod tolerance;
mod validation;
mod values;
pub use bounds::hatch_bounds;
pub use gradient::*;
pub use pattern::*;
pub use tolerance::resolve_hatch_join_tolerance;
pub use tolerance::{HatchJoinToleranceError, HatchJoinToleranceRequest};
pub use validation::validate_hatch_boundaries;
pub use values::*;
