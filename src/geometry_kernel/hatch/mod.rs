//! ID-free planar Hatch parameters, join proofs and conservative enclosures.
mod bounds;
mod endpoints;
mod tolerance;
mod validation;
mod values;
pub use bounds::hatch_bounds;
pub use tolerance::resolve_hatch_join_tolerance;
pub use tolerance::{HatchJoinToleranceError, HatchJoinToleranceRequest};
pub use validation::validate_hatch_boundaries;
pub use values::*;
