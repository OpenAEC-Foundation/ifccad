use crate::geometry_kernel::{SplineControl, SplineError, SplineKnots};
pub use crate::geometry_kernel::{
    SplineFitData as IfccadSplineFitData, SplineParameterization as IfccadSplineParameterization,
};
use crate::IfccadReport;
use serde::{Deserialize, Serialize};

/// A standalone spline in its owner's XYZ coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadSpline {
    pub definition: IfccadSplineDefinition,
    pub topology: IfccadSplineTopology,
    pub normal: Option<[f64; 3]>,
    pub planar: Option<bool>,
    pub linear: Option<bool>,
    pub rational: Option<bool>,
    pub knot_tolerance: Option<f64>,
    pub control_tolerance: Option<f64>,
    pub authoring_mode: Option<IfccadSplineAuthoringMode>,
    pub control_frame_visible: Option<bool>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum IfccadSplineDefinition {
    Control {
        basis: SplineControl<3>,
        fit: Option<IfccadSplineFitData<3>>,
    },
    FitOnly {
        degree: Option<u32>,
        fit: IfccadSplineFitData<3>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadSplineTopology {
    Open,
    Closed,
    Periodic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadSplineAuthoringMode {
    Control,
    Fit,
}

impl IfccadSpline {
    pub fn new(topology: IfccadSplineTopology, definition: IfccadSplineDefinition) -> Self {
        Self {
            topology,
            definition,
            normal: None,
            planar: None,
            linear: None,
            rational: None,
            knot_tolerance: None,
            control_tolerance: None,
            authoring_mode: None,
            control_frame_visible: None,
        }
    }

    pub fn control_basis(&self) -> Option<&SplineControl<3>> {
        match &self.definition {
            IfccadSplineDefinition::Control { basis, .. } => Some(basis),
            IfccadSplineDefinition::FitOnly { .. } => None,
        }
    }

    pub fn validate(&self) -> Result<(), IfccadReport> {
        self.validate_at("/ifccad::geom::spline")
    }

    pub(super) fn validate_at(&self, path: &str) -> Result<(), IfccadReport> {
        let error = |reason: &str| {
            crate::ifccad::diagnostics::failure("IFCCAD-SPLINE-001", path, reason.to_owned())
        };
        if !valid_tolerance(self.knot_tolerance) || !valid_tolerance(self.control_tolerance) {
            return Err(error("spline tolerances must be finite and nonnegative"));
        }
        if self
            .normal
            .is_some_and(|normal| normal.iter().any(|x| !x.is_finite()) || normal == [0.; 3])
            || (self.planar == Some(false) && (self.normal.is_some() || self.linear == Some(true)))
        {
            return Err(error("invalid or contradictory spline plane data"));
        }
        let fit = match &self.definition {
            IfccadSplineDefinition::Control { basis, fit } => {
                if matches!(basis.knots, SplineKnots::PeriodicCycle(_))
                    && self.topology != IfccadSplineTopology::Periodic
                {
                    return Err(error("a cyclic basis requires periodic topology"));
                }
                if self.rational == Some(false)
                    && basis
                        .weights
                        .as_ref()
                        .is_some_and(|w| w.iter().any(|x| *x != 1.))
                {
                    return Err(error("non-unit weights contradict non-rational geometry"));
                }
                match basis.prepare() {
                    Ok(_) | Err(SplineError::EvaluationUnavailable) => {}
                    Err(reason) => {
                        return Err(error(&format!("invalid control basis: {reason:?}")))
                    }
                }
                fit.as_ref()
            }
            IfccadSplineDefinition::FitOnly { degree, fit } => {
                if *degree == Some(0) {
                    return Err(error("spline degree must be positive"));
                }
                Some(fit)
            }
        };
        if fit.is_some_and(|fit| !fit.valid()) {
            return Err(error("invalid spline fit data"));
        }
        Ok(())
    }
}

fn valid_tolerance(value: Option<f64>) -> bool {
    value.is_none_or(|v| v.is_finite() && v >= 0.)
}
