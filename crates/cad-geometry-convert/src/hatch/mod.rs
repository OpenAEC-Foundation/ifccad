//! Stored Hatch contours/families and independent numerical conversion evidence.
mod audit;
mod evidence;
mod from_cad;
mod gradient;
mod loop_flags;
pub use gradient::{apply_gradient_to_cad, prepare_gradient_from_cad, PreparedNativeGradient};
pub use loop_flags::{
    apply_loop_flags_to_cad, prepare_loop_flags_from_cad, PreparedHatchLoopFlags,
};
pub mod pattern;
mod to_cad;
use crate::{CadPreparationError, GeometryPair};
pub use from_cad::prepare_hatch_from_cad;
pub use from_cad::prepare_hatch_from_cad_with_pattern_context;
use ifccad::geometry_kernel::{
    hatch::{HatchAreaRule, HatchBoundary2, HatchFill, HatchValidationError},
    CoordinateFrame3,
};
use opencadcodec::{entities::Hatch, Handle};
pub use to_cad::prepare_hatch_to_cad;
/// Resolve a producer limit before a native document exists. `Some(None)`
/// denotes Paper coordinates without a known physical mapping.
pub fn resolve_creation_tolerance(
    request: ifccad::geometry_kernel::hatch::HatchJoinToleranceRequest,
    unit: &str,
    paper: Option<Option<&ifccad::plot_kernel::PlotSettings>>,
) -> Result<f64, ifccad::geometry_kernel::hatch::HatchJoinToleranceError> {
    use ifccad::{
        geometry_kernel::{
            hatch::{resolve_hatch_join_tolerance, HatchJoinToleranceError},
            CoordinateLengthUnit,
        },
        plot_kernel::PlotScale,
    };
    let metres = if let Some(plot) = paper {
        if let Some(p) = plot {
            if let PlotScale::Fixed {
                output_length,
                scope_length,
            } = p.mapping.scale
            {
                if !output_length.is_finite()
                    || !scope_length.is_finite()
                    || output_length <= 0.
                    || scope_length <= 0.
                {
                    return Err(HatchJoinToleranceError::InvalidValue);
                }
            }
        }
        match crate::plot_units::paper_mapping(plot) {
            crate::plot_units::PaperMapping::Unknown => None,
            crate::plot_units::PaperMapping::FixedPhysical {
                metres_per_coordinate,
            } => Some(metres_per_coordinate),
        }
    } else {
        CoordinateLengthUnit::from_token(unit)
            .ok_or(HatchJoinToleranceError::InvalidValue)?
            .coordinates_per_metre()
            .map(|(lower, _)| lower.recip())
    };
    resolve_hatch_join_tolerance(request, metres.as_ref())
}

pub struct PreparedNativeHatch {
    pub loop_flags: Vec<PreparedHatchLoopFlags>,
    pub gradient: Option<ifccad::geometry_kernel::hatch::HatchGradient<opencadcodec::Color>>,
    pub fill: HatchFill,
    pub placement: CoordinateFrame3,
    pub boundaries: Vec<HatchBoundary2>,
    pub area_rule: HatchAreaRule,
    pub join_tolerance: f64,
    pub source_handles: Vec<Vec<Handle>>,
    pub is_associative: bool,
    pub pairs: Vec<GeometryPair>,
    pub losses: Vec<HatchSourceLoss>,
}
pub struct PreparedCadHatch {
    pub hatch: Hatch,
    pub pairs: Vec<GeometryPair>,
    pub losses: Vec<HatchSourceLoss>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HatchSourceLoss {
    pub field: &'static str,
    pub detail: String,
}
#[derive(Debug, thiserror::Error)]
pub enum CadHatchPreparationError {
    #[error("Hatch loop count mismatch: expected {expected}, got {actual}")]
    LoopCountMismatch { expected: usize, actual: usize },
    #[error("{0}")]
    Gradient(#[from] ifccad::geometry_kernel::hatch::HatchGradientValidationError),
    #[error("invalid gradient {field}: {detail}")]
    InvalidGradientSource { field: &'static str, detail: String },
    #[error("{0}")]
    Primitive(#[from] CadPreparationError),
    #[error("{0}")]
    Boundary(#[from] HatchValidationError),
    #[error("unsupported Hatch {field}: {detail}")]
    Unsupported { field: &'static str, detail: String },
}
fn unsupported(field: &'static str, detail: impl Into<String>) -> CadHatchPreparationError {
    CadHatchPreparationError::Unsupported {
        field,
        detail: detail.into(),
    }
}
