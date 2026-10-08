//! Identifier-free saved drawing workspace values and intrinsic validation.
//! Coordinate meaning belongs to the consuming workspace; no CAD runtime is required.
mod aids;
mod validation;
mod view;
pub use aids::*;
pub use validation::*;
pub use view::*;
