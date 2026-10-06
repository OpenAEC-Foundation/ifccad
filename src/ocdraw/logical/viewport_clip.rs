//! Active boundary eligibility shared with CAD conversion.
use super::{DrawingViewportFrame, EntityGeometry, LogicalError, OcdrawValidationError};

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
    let Some(geometry) = super::geometry_validation::geometry_ref(boundary) else {
        return Err(fail(
            "VIEWPORT_CLIP",
            "/boundary",
            "active boundary requires circle, full ellipse or closed planar geometry",
        ));
    };
    crate::geometry_kernel::validate_paper_boundary(
        crate::geometry_kernel::PaperFrame {
            center: [frame.center.x(), frame.center.y()],
            width: frame.width,
            height: frame.height,
        },
        geometry,
    )
    .map_err(|error| fail("VIEWPORT_CLIP", "/boundary", &error.to_string()))
}
