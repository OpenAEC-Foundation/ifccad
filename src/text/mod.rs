//! Format-independent authored text values and intrinsic validation.

mod extent;
mod inheritance;
mod mtext_extent;
mod tabs;
mod validation;
mod values;

pub use extent::*;
pub use inheritance::*;
pub use mtext_extent::estimate_mtext_extent;
pub use tabs::next_tab_stop;
pub use validation::*;
pub use values::*;

#[cfg(test)]
mod tests;
