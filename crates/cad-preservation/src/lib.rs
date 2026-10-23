//! Codec-owned byte snapshots, independent from drawing formats.
mod common_snapshot;
mod spline_snapshot;
pub use spline_snapshot::{capture_spline, decode_spline_snapshot, CadSplineSnapshot};
pub const SPLINE_SCHEMA: &str = "openaec.opencadcodec.spline";
pub const CODEC_REVISION: &str = "bd10efc2c54dfe1e6288c5f10486dce9be0d7f32";
pub const OCTOBER_16_CODEC_REVISION: &str = "325fa98d2b881f48e106a830d5cf161624f63131";
pub const OCTOBER_8_CODEC_REVISION: &str = "ab2eecdbffc31120b5ad6d899f6fc67cf21ede39";
pub const PREVIOUS_CODEC_REVISION: &str = "063c10671fe7833d562f772159771318c7a0ebb9";
pub const LEGACY_CODEC_REVISION: &str = "fe69506cb99dea6f4c4a73b690a27fdf04403ea0";
pub const SPLINE_PAYLOAD_VERSION: u32 = 3;

/// Payload shape audited for an exact source codec revision.
pub fn spline_payload_version(revision: &str) -> Option<u32> {
    match revision {
        CODEC_REVISION | OCTOBER_16_CODEC_REVISION => Some(3),
        OCTOBER_8_CODEC_REVISION | PREVIOUS_CODEC_REVISION => Some(2),
        LEGACY_CODEC_REVISION => Some(1),
        _ => None,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CadSplineSnapshotError {
    #[error("malformed spline source snapshot: {0}")]
    Malformed(serde_json::Error),
    #[error("spline snapshot codec revision has not been audited")]
    UnsupportedRevision,
    #[error("spline snapshot source identities disagree")]
    InconsistentIdentity,
}

#[cfg(test)]
mod tests;
