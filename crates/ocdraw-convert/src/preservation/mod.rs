mod conditions;
mod references;
mod report;
mod restore;
pub(crate) use conditions::{
    build_spline_conditions, evaluate_spline_conditions, record_unassessed_occurrences,
    unsupported_common_context,
};
pub(crate) use references::{
    bind_source_references, qualify_references, rebind_spline_references, target_exists,
    validate_reference_condition,
};
pub use report::*;
pub(crate) use restore::{report_restore, restore_spline};
#[cfg(test)]
mod tests;
pub use cad_preservation::CadSplineSnapshotError as OcdrawSplineSnapshotError;
pub(crate) use cad_preservation::{
    capture_spline, decode_spline_snapshot, CODEC_REVISION, LEGACY_CODEC_REVISION,
    PREVIOUS_CODEC_REVISION, SPLINE_PAYLOAD_VERSION,
};
