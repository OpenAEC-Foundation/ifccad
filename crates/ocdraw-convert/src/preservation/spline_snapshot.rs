use super::common_snapshot::{SnapshotFloat, SnapshotHandle, SnapshotVector, SplineCommonSnapshot};
use super::{OcdrawSplineSnapshotError, CODEC_REVISION, LEGACY_CODEC_REVISION};
use opencadcodec::entities::{Spline, SplineFlags};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SnapshotFlags {
    closed: bool,
    periodic: bool,
    rational: bool,
    planar: bool,
    linear: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SplineSnapshot {
    pub codec_revision: String,
    pub source_handle: SnapshotHandle,
    pub source_owner_handle: SnapshotHandle,
    pub source_order_index: u64,
    pub common: SplineCommonSnapshot,
    pub flags: SnapshotFlags,
    pub degree: i32,
    pub knots: Vec<SnapshotFloat>,
    pub control_points: Vec<SnapshotVector>,
    pub weights: Vec<SnapshotFloat>,
    pub fit_points: Vec<SnapshotVector>,
    pub normal: SnapshotVector,
    pub begin_tangent: SnapshotVector,
    pub end_tangent: SnapshotVector,
    pub knot_tolerance: SnapshotFloat,
    pub control_tolerance: SnapshotFloat,
    pub fit_tolerance: SnapshotFloat,
    pub knot_parameterization: i32,
    pub cv_frame_visible: bool,
    pub dwg_flags1: i32,
    #[serde(default)]
    pub dwg_scenario: Option<i32>,
    pub dxf_flags: i16,
}
impl SplineSnapshot {
    fn capture(source: &Spline, order_index: u64, codec_revision: &str) -> Self {
        let Spline {
            common,
            flags,
            degree,
            knots,
            control_points,
            weights,
            fit_points,
            normal,
            begin_tangent,
            end_tangent,
            knot_tolerance,
            control_tolerance,
            fit_tolerance,
            knot_parameterization,
            cv_frame_visible,
            dwg_flags1,
            dwg_scenario,
            dxf_flags,
        } = source;
        let SplineFlags {
            closed,
            periodic,
            rational,
            planar,
            linear,
        } = *flags;
        Self {
            codec_revision: codec_revision.into(),
            source_handle: SnapshotHandle(common.handle),
            source_owner_handle: SnapshotHandle(common.owner_handle),
            source_order_index: order_index,
            common: SplineCommonSnapshot::capture(common),
            flags: SnapshotFlags {
                closed,
                periodic,
                rational,
                planar,
                linear,
            },
            degree: *degree,
            knots: knots.iter().copied().map(SnapshotFloat).collect(),
            control_points: control_points
                .iter()
                .copied()
                .map(SnapshotVector::capture)
                .collect(),
            weights: weights.iter().copied().map(SnapshotFloat).collect(),
            fit_points: fit_points
                .iter()
                .copied()
                .map(SnapshotVector::capture)
                .collect(),
            normal: SnapshotVector::capture(*normal),
            begin_tangent: SnapshotVector::capture(*begin_tangent),
            end_tangent: SnapshotVector::capture(*end_tangent),
            knot_tolerance: SnapshotFloat(*knot_tolerance),
            control_tolerance: SnapshotFloat(*control_tolerance),
            fit_tolerance: SnapshotFloat(*fit_tolerance),
            knot_parameterization: *knot_parameterization,
            cv_frame_visible: *cv_frame_visible,
            dwg_flags1: *dwg_flags1,
            dwg_scenario: *dwg_scenario,
            dxf_flags: *dxf_flags,
        }
    }
    pub fn to_source(&self) -> Spline {
        Spline {
            common: self.common.restore(),
            flags: SplineFlags {
                closed: self.flags.closed,
                periodic: self.flags.periodic,
                rational: self.flags.rational,
                planar: self.flags.planar,
                linear: self.flags.linear,
            },
            degree: self.degree,
            knots: self.knots.iter().map(|v| v.0).collect(),
            control_points: self.control_points.iter().map(|v| v.restore()).collect(),
            weights: self.weights.iter().map(|v| v.0).collect(),
            fit_points: self.fit_points.iter().map(|v| v.restore()).collect(),
            normal: self.normal.restore(),
            begin_tangent: self.begin_tangent.restore(),
            end_tangent: self.end_tangent.restore(),
            knot_tolerance: self.knot_tolerance.0,
            control_tolerance: self.control_tolerance.0,
            fit_tolerance: self.fit_tolerance.0,
            knot_parameterization: self.knot_parameterization,
            cv_frame_visible: self.cv_frame_visible,
            dwg_flags1: self.dwg_flags1,
            dwg_scenario: self.dwg_scenario,
            dxf_flags: self.dxf_flags,
        }
    }
}

pub(crate) fn capture_spline(
    source: &Spline,
    order_index: u64,
    codec_revision: &str,
) -> Result<Vec<u8>, OcdrawSplineSnapshotError> {
    if codec_revision != CODEC_REVISION {
        return Err(OcdrawSplineSnapshotError::UnsupportedRevision);
    }
    serde_json::to_vec(&SplineSnapshot::capture(
        source,
        order_index,
        codec_revision,
    ))
    .map_err(OcdrawSplineSnapshotError::Malformed)
}
pub(crate) fn decode_spline_snapshot(
    bytes: &[u8],
) -> Result<SplineSnapshot, OcdrawSplineSnapshotError> {
    let snapshot: SplineSnapshot =
        serde_json::from_slice(bytes).map_err(OcdrawSplineSnapshotError::Malformed)?;
    if snapshot.codec_revision != CODEC_REVISION && snapshot.codec_revision != LEGACY_CODEC_REVISION
    {
        return Err(OcdrawSplineSnapshotError::UnsupportedRevision);
    }
    // Version 1 predates these source fields. Version 2 must state both,
    // including explicit null; accepting omission would invent source state.
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(OcdrawSplineSnapshotError::Malformed)?;
    let scenario = value.get("dwgScenario").is_some();
    let layer_handle = value["common"].get("layerHandle").is_some();
    let valid_shape = if snapshot.codec_revision == CODEC_REVISION {
        scenario && layer_handle
    } else {
        !scenario && !layer_handle
    };
    if !valid_shape {
        return Err(OcdrawSplineSnapshotError::Malformed(
            <serde_json::Error as serde::de::Error>::custom(
                "source field presence does not match the audited codec revision",
            ),
        ));
    }
    if snapshot.source_handle.0 != snapshot.common.handle.0
        || snapshot.source_owner_handle.0 != snapshot.common.owner_handle.0
    {
        return Err(OcdrawSplineSnapshotError::InconsistentIdentity);
    }
    Ok(snapshot)
}
