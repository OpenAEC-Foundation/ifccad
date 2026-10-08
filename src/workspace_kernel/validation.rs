use super::*;
use num_rational::BigRational;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceViewKind {
    Model,
    PaperCanvas,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceScalarReason {
    NonFinite,
    OutOfRange,
    InvalidView,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid workspace field {field}: {reason:?}")]
pub struct WorkspaceScalarError {
    pub field: &'static str,
    pub reason: WorkspaceScalarReason,
}

fn error(field: &'static str, reason: WorkspaceScalarReason) -> WorkspaceScalarError {
    WorkspaceScalarError { field, reason }
}
fn finite(
    field: &'static str,
    values: impl IntoIterator<Item = f64>,
) -> Result<(), WorkspaceScalarError> {
    if values.into_iter().all(f64::is_finite) {
        Ok(())
    } else {
        Err(error(field, WorkspaceScalarReason::NonFinite))
    }
}

pub fn validate_grid(grid: &WorkspaceGrid) -> Result<(), WorkspaceScalarError> {
    let spacing = [grid.spacing.x(), grid.spacing.y()];
    finite("grid.spacing", spacing)?;
    if spacing.into_iter().any(|v| v < 0.) {
        return Err(error("grid.spacing", WorkspaceScalarReason::OutOfRange));
    }
    if grid.major_line_frequency == 0 {
        return Err(error(
            "grid.majorLineFrequency",
            WorkspaceScalarReason::OutOfRange,
        ));
    }
    Ok(())
}

pub fn validate_snap(snap: &WorkspaceSnap) -> Result<(), WorkspaceScalarError> {
    finite("snap.base", [snap.base.x(), snap.base.y()])?;
    finite("snap.angle", [snap.angle])?;
    let spacing = [snap.spacing.x(), snap.spacing.y()];
    finite("snap.spacing", spacing)?;
    if spacing
        .into_iter()
        .any(|v| v < 0. || (snap.enabled && v == 0.))
    {
        return Err(error("snap.spacing", WorkspaceScalarReason::OutOfRange));
    }
    Ok(())
}

pub fn validate_canvas_frame(frame: &WorkspaceCanvasFrame) -> Result<(), WorkspaceScalarError> {
    finite("frame.center", frame.center.components())?;
    finite("frame.width", [frame.width])?;
    finite("frame.height", [frame.height])?;
    if frame.width <= 0. || frame.height <= 0. {
        return Err(error("frame.dimensions", WorkspaceScalarReason::OutOfRange));
    }
    Ok(())
}

pub fn validate_view(
    view: &WorkspaceView,
    kind: WorkspaceViewKind,
) -> Result<(), WorkspaceScalarError> {
    finite("view.center", [view.center.x(), view.center.y()])?;
    finite("view.target", view.target.components())?;
    let direction = view.direction.components();
    finite("view.direction", direction)?;
    finite("view.height", [view.height])?;
    finite("view.twist", [view.twist])?;
    let exact = |v: f64| BigRational::from_float(v).expect("finite workspace scalar");
    let norm_squared: BigRational = direction
        .into_iter()
        .map(|v| {
            let q = exact(v);
            &q * &q
        })
        .sum();
    let maximum = exact(f64::MAX);
    if norm_squared <= exact(0.) || norm_squared > &maximum * &maximum {
        return Err(error("view.direction", WorkspaceScalarReason::OutOfRange));
    }
    if view.height <= 0. {
        return Err(error("view.height", WorkspaceScalarReason::OutOfRange));
    }
    if kind == WorkspaceViewKind::PaperCanvas
        && view.projection != WorkspaceProjection::Orthographic
    {
        return Err(error("view.projection", WorkspaceScalarReason::InvalidView));
    }
    if let Some(value) = view.lens_length {
        finite("view.lensLength", [value])?;
    }
    if !match (view.projection, view.lens_length) {
        (WorkspaceProjection::Orthographic, None) => true,
        (WorkspaceProjection::Orthographic, Some(v)) => v >= 0.,
        (WorkspaceProjection::Perspective, Some(v)) => v > 0.,
        _ => false,
    } {
        return Err(error("view.lensLength", WorkspaceScalarReason::OutOfRange));
    }
    if let Some(value) = view.front_clip.distance {
        finite("view.frontClip.distance", [value])?;
    }
    if let Some(value) = view.back_clip.distance {
        finite("view.backClip.distance", [value])?;
    }
    if view.front_clip.mode == WorkspaceClipMode::AtDistance && view.front_clip.distance.is_none() {
        return Err(error(
            "view.frontClip.distance",
            WorkspaceScalarReason::InvalidView,
        ));
    }
    if view.back_clip.mode == WorkspaceClipMode::AtCamera
        || (view.back_clip.mode == WorkspaceClipMode::AtDistance
            && view.back_clip.distance.is_none())
    {
        return Err(error("view.backClip", WorkspaceScalarReason::InvalidView));
    }
    if view.front_clip.mode != WorkspaceClipMode::Disabled
        && view.back_clip.mode != WorkspaceClipMode::Disabled
    {
        let back = view.back_clip.distance.expect("validated active distance");
        let ordered = match view.front_clip.mode {
            WorkspaceClipMode::AtCamera => {
                back < 0. || {
                    let q = exact(back);
                    &q * &q < norm_squared
                }
            }
            WorkspaceClipMode::AtDistance => {
                back < view.front_clip.distance.expect("validated active distance")
            }
            WorkspaceClipMode::Disabled => unreachable!(),
        };
        if !ordered {
            return Err(error("view.clipPlanes", WorkspaceScalarReason::InvalidView));
        }
    }
    Ok(())
}
