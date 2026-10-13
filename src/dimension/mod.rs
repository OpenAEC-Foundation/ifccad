//! Native dimension meaning and pure measurement; independent of CAD and fonts.
mod formatting;
mod measurement;
mod references;
mod styles;
mod validation;
mod values;
pub use formatting::*;
pub use measurement::*;
pub use styles::*;
pub use validation::*;
pub use values::*;

/// Omission means inheritance. An explicit JSON null is never an override.
pub(crate) fn present_option<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    T::deserialize(d).map(Some)
}
