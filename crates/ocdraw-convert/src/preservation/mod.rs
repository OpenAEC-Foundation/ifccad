mod common_snapshot;
mod conditions;
mod references;
mod report;
mod restore;
pub(crate) use references::{
    bind_source_references, qualify_references, rebind_spline_references, target_exists,
    validate_reference_condition,
};
mod spline_snapshot;
pub(crate) use conditions::{
    build_spline_conditions, evaluate_spline_conditions, record_unassessed_occurrences,
    unsupported_common_context,
};
pub use report::*;
pub(crate) use restore::{report_restore, restore_spline};
#[cfg(test)]
mod tests;
pub(crate) use spline_snapshot::{capture_spline, decode_spline_snapshot};
pub(crate) const CODEC_REVISION: &str = "fe69506cb99dea6f4c4a73b690a27fdf04403ea0";

#[derive(Debug, thiserror::Error)]
pub enum OcdrawSplineSnapshotError {
    #[error("malformed spline source snapshot: {0}")]
    Malformed(serde_json::Error),
    #[error("spline snapshot codec revision has not been audited")]
    UnsupportedRevision,
    #[error("spline snapshot source identities disagree")]
    InconsistentIdentity,
}
