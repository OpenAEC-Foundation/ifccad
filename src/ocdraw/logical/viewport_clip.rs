//! Active boundary eligibility shared with CAD conversion.
use super::{DrawingViewportFrame, EntityGeometry, LogicalError, OcdrawValidationError};
use std::collections::BTreeSet;

/// Checks an active paper boundary's geometry, closure, plane and full-curve containment.
///
/// Reference ownership and uniqueness require [`super::validate_ocdraw_document`].
/// Disabled clip references do not require active boundary eligibility.
///
/// ```
/// use ocdraw::ocdraw::*;
/// let frame = DrawingViewportFrame { center: Point2::new(0., 0.), width: 4., height: 4. };
/// let boundary = DrawingGeometry::Circle { placement: CoordinateFrame3::default(), radius: 2. };
/// validate_viewport_clip_boundary(frame, &boundary)?;
/// # Ok::<(), OcdrawValidationError>(())
/// ```
pub fn validate_viewport_clip_boundary(
    frame: DrawingViewportFrame,
    boundary: &EntityGeometry,
) -> Result<(), OcdrawValidationError> {
    let fail = |code, location, message: &str| {
        OcdrawValidationError::from_logical_errors(vec![LogicalError {
            code,
            location: String::from(location),
            message: message.into(),
        }])
    };
    if super::viewport_bounds(frame).is_none() {
        return Err(fail(
            "VIEWPORT_FRAME",
            "/frame",
            "clip frame requires positive finite dimensions and finite enclosure",
        ));
    }
    let supported = match boundary {
        EntityGeometry::Circle { .. } | EntityGeometry::Ellipse { arc: None, .. } => true,
        EntityGeometry::PlanarPolyline {
            vertices,
            closed: true,
            ..
        } => {
            let distinct = vertices
                .iter()
                .map(|v| {
                    let bits = |x: f64| if x == 0. { 0 } else { x.to_bits() };
                    (bits(v[0]), bits(v[1]))
                })
                .collect::<BTreeSet<_>>()
                .len();
            let curved = vertices.iter().any(|v| v[2] != 0.);
            distinct >= if curved { 2 } else { 3 }
        }
        _ => false,
    };
    if !supported || !super::field_validation::geometry_is_valid(boundary) {
        return Err(fail("VIEWPORT_CLIP","/boundary","active boundary requires valid circle, full ellipse or closed planar polyline geometry"));
    }
    if !crate::ocdraw::geometry::clip_containment::enclosed_by_frame(frame, boundary) {
        return Err(fail(
            "VIEWPORT_CLIP",
            "/boundary",
            "entire active boundary must lie in paper Z=0 inside the viewport frame",
        ));
    }
    Ok(())
}
