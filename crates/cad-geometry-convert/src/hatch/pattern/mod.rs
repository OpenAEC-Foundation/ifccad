//! Explicit family mapping; geometry is evaluated once and certified separately.
pub(super) mod audit;
mod evidence;
pub use audit::residual_common;
pub(super) use evidence::double_evidence;
mod math;
mod projection;
use super::{CadHatchPreparationError, HatchSourceLoss};
use crate::{CadPreparationError, GeometryPair};
use ocdraw::geometry_kernel::{hatch::*, CoordinateFrame3};
use opencadcodec::{
    entities::{Hatch, HatchPattern},
    Vector2, Vector3,
};
pub(super) use projection::validate_source_scalars;
pub struct PreparedNativePattern {
    pub pattern: HatchLinePattern,
    pub pair: GeometryPair,
    pub losses: Vec<HatchSourceLoss>,
}
pub struct PreparedCadPattern {
    pub pattern: HatchPattern,
    pub angle: f64,
    pub scale: f64,
    pub origin: Vector2,
    pub pair: GeometryPair,
    pub losses: Vec<HatchSourceLoss>,
}
/// Map already-effective literal families. Full Hatch activation (gradient,
/// double, annotation and linetype context) is audited by Hatch preparation.
pub fn prepare_pattern_from_cad(
    h: &Hatch,
    plane: CoordinateFrame3,
    boundaries: &[HatchBoundary2],
) -> Result<PreparedNativePattern, CadHatchPreparationError> {
    validate_plane(h.normal, h.elevation)?;
    residual_common(h)?;
    let p = projection::from_cad(h)?;
    let pair = evidence::from_cad(h, &p, plane, boundaries)?;
    Ok(PreparedNativePattern {
        pattern: p,
        pair,
        losses: vec![],
    })
}
/// Evaluate literal families once in the supplied target OCS and certify them
/// over both contour enclosures. This does not generate or clip fill geometry.
pub fn prepare_pattern_to_cad(
    p: &HatchLinePattern,
    plane: CoordinateFrame3,
    normal: Vector3,
    elevation: f64,
    boundaries: &[HatchBoundary2],
) -> Result<PreparedCadPattern, CadHatchPreparationError> {
    validate_plane(normal, elevation)?;
    validate_hatch_fill(&HatchFill::LinePattern(p.clone()))
        .map_err(|e| super::unsupported("pattern", e.to_string()))?;
    let (pattern, angle, origin) = projection::to_cad(p, plane, normal)?;
    let mut h = super::to_cad::construct_hatch(plane, boundaries, HatchAreaRule::Normal)?;
    h.pattern = pattern.clone();
    h.is_solid = false;
    h.normal = normal;
    h.elevation = elevation;
    h.pattern_angle = angle;
    h.pattern_scale = p.scale;
    h.record_pattern_origin(origin);
    let pair = evidence::to_cad(p, &h, plane, boundaries)?;
    Ok(PreparedCadPattern {
        pattern,
        angle,
        scale: p.scale,
        origin,
        pair,
        losses: [
            (
                p.name.is_none(),
                "pattern.name",
                "CAD requires a literal name; absent native metadata is written as OCDRAW_PATTERN",
            ),
            (
                p.description.is_none(),
                "pattern.description",
                "CAD writes absent native description as an empty literal",
            ),
        ]
        .into_iter()
        .filter(|v| v.0)
        .map(|(_, field, detail)| HatchSourceLoss {
            field,
            detail: detail.into(),
        })
        .collect(),
    })
}

fn validate_plane(normal: Vector3, elevation: f64) -> Result<(), CadHatchPreparationError> {
    if ![normal.x, normal.y, normal.z, elevation]
        .into_iter()
        .all(f64::is_finite)
        || crate::geometry::cad_plane(normal).is_none()
    {
        return Err(CadPreparationError::InvalidGeometry.into());
    }
    Ok(())
}
