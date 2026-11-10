use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
pub struct SplineFitData<const D: usize> {
    pub points: Vec<[f64; D]>,
    pub start_tangent: Option<[f64; D]>,
    pub end_tangent: Option<[f64; D]>,
    pub tolerance: Option<f64>,
    pub parameterization: Option<SplineParameterization>,
}

impl<const D: usize> SplineFitData<D> {
    pub(crate) fn valid(&self) -> bool {
        self.points.len() >= 2
            && self.points.iter().flatten().all(|x| x.is_finite())
            && self
                .start_tangent
                .iter()
                .chain(&self.end_tangent)
                .flatten()
                .all(|x| x.is_finite())
            && self.tolerance.is_none_or(|v| v.is_finite() && v >= 0.)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SplineParameterization {
    Chord,
    Centripetal,
    Uniform,
    Custom,
}

/// Knot data for a full basis or one authored period.
#[derive(Clone, Debug, PartialEq)]
pub enum SplineKnots {
    Full(Vec<f64>),
    PeriodicCycle(Vec<f64>),
}

/// Shared control-point basis in local coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct SplineControl<const D: usize> {
    pub degree: usize,
    pub knots: SplineKnots,
    pub points: Vec<[f64; D]>,
    pub weights: Option<Vec<f64>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplineLimit {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplineError {
    ControlCount,
    KnotCount,
    InvalidKnots,
    EmptyDomain,
    InvalidPoints,
    InvalidWeights,
    DiscontinuousSeam,
    ParameterOutsideDomain,
    EvaluationUnavailable,
}

/// Validated evaluation data. Expanded periodic knots are derived here.
pub struct PreparedSpline<'a, const D: usize> {
    control: &'a SplineControl<D>,
    knots: Cow<'a, [f64]>,
    control_count: usize,
}

impl<const D: usize> SplineControl<D> {
    /// Expand a compact cycle for a target requiring an explicit full basis.
    /// Every derived knot must have an exact binary64 representation.
    pub fn exact_periodic_expansion(&self) -> Result<Option<Self>, SplineError> {
        let SplineKnots::PeriodicCycle(cycle) = &self.knots else {
            return Ok(None);
        };
        self.prepare()?;
        let degree = self.degree;
        let exact = super::spline_exact::periodic_knots(cycle, degree);
        let mut knots = Vec::with_capacity(exact.len());
        for k in &exact {
            let lo =
                super::numeric::round_down(k).map_err(|_| SplineError::EvaluationUnavailable)?;
            let hi = super::numeric::round_up(k).map_err(|_| SplineError::EvaluationUnavailable)?;
            if lo != hi {
                return Err(SplineError::EvaluationUnavailable);
            }
            knots.push(lo);
        }
        knots[degree..degree + cycle.len()].copy_from_slice(cycle);
        let mut points = self.points.clone();
        points.extend_from_slice(&self.points[..degree]);
        let mut weights = self.weights.clone();
        if let (Some(weights), Some(source)) = (&mut weights, &self.weights) {
            weights.extend_from_slice(&source[..degree]);
        }
        Ok(Some(Self {
            degree,
            knots: SplineKnots::Full(knots),
            points,
            weights,
        }))
    }

    pub fn prepare(&self) -> Result<PreparedSpline<'_, D>, SplineError> {
        let count = self.points.len();
        let degree = self.degree;
        if degree == 0 || count == 0 {
            return Err(SplineError::ControlCount);
        }
        if self.points.iter().flatten().any(|v| !v.is_finite()) {
            return Err(SplineError::InvalidPoints);
        }
        if self.weights.as_ref().is_some_and(|weights| {
            weights.len() != count || weights.iter().any(|w| !w.is_finite() || *w <= 0.)
        }) {
            return Err(SplineError::InvalidWeights);
        }
        let (knots, control_count) = match &self.knots {
            SplineKnots::Full(knots) => {
                if degree >= count {
                    return Err(SplineError::ControlCount);
                }
                (Cow::Borrowed(knots.as_slice()), count)
            }
            SplineKnots::PeriodicCycle(cycle) => {
                if degree > count {
                    return Err(SplineError::ControlCount);
                }
                if count.checked_add(1) != Some(cycle.len()) {
                    return Err(SplineError::KnotCount);
                }
                validate_knots(cycle, degree)?;
                let period = cycle[count] - cycle[0];
                if period <= 0. {
                    return Err(SplineError::EmptyDomain);
                }
                if !period.is_finite() {
                    return Err(SplineError::EvaluationUnavailable);
                }
                let mut knots = Vec::new();
                let expanded_count = degree
                    .checked_mul(2)
                    .and_then(|extra| cycle.len().checked_add(extra))
                    .ok_or(SplineError::KnotCount)?;
                knots
                    .try_reserve_exact(expanded_count)
                    .map_err(|_| SplineError::EvaluationUnavailable)?;
                knots.extend(cycle[count - degree..count].iter().map(|k| *k - period));
                knots.extend_from_slice(cycle);
                knots.extend(cycle[1..=degree].iter().map(|k| *k + period));
                (Cow::Owned(knots), count + degree)
            }
        };
        if control_count
            .checked_add(degree)
            .and_then(|length| length.checked_add(1))
            != Some(knots.len())
        {
            return Err(SplineError::KnotCount);
        }
        validate_knots(&knots, degree)?;
        let prepared = PreparedSpline {
            control: self,
            knots,
            control_count,
        };
        let [start, end] = prepared.domain();
        if start >= end {
            return Err(SplineError::EmptyDomain);
        }
        if matches!(self.knots, SplineKnots::PeriodicCycle(_))
            && prepared.knots.iter().filter(|&&k| k == start).count() > degree
            && prepared.evaluate(start)? != prepared.evaluate(end)?
        {
            return Err(SplineError::DiscontinuousSeam);
        }
        Ok(prepared)
    }
}

fn validate_knots(knots: &[f64], degree: usize) -> Result<(), SplineError> {
    if knots.iter().any(|k| !k.is_finite())
        || knots.windows(2).any(|pair| pair[0] > pair[1])
        || knots
            .windows(degree + 2)
            .any(|run| run[0] == run[run.len() - 1])
    {
        Err(SplineError::InvalidKnots)
    } else {
        Ok(())
    }
}

impl<const D: usize> PreparedSpline<'_, D> {
    pub fn domain(&self) -> [f64; 2] {
        [
            self.knots[self.control.degree],
            self.knots[self.control_count],
        ]
    }

    /// Evaluate using the right limit at an interior knot and the left limit at the end.
    pub fn evaluate(&self, parameter: f64) -> Result<[f64; D], SplineError> {
        self.evaluate_limit(parameter, SplineLimit::Right)
    }

    pub fn evaluate_limit(
        &self,
        parameter: f64,
        side: SplineLimit,
    ) -> Result<[f64; D], SplineError> {
        let [start, end] = self.domain();
        if !parameter.is_finite() || parameter < start || parameter > end {
            return Err(SplineError::ParameterOutsideDomain);
        }
        let right = parameter == start || (parameter < end && side == SplineLimit::Right);
        let span = self
            .knots
            .partition_point(|&k| k < parameter || (right && k == parameter))
            - 1;
        let degree = self.control.degree;
        let first = span - degree;
        let count = self.control.points.len();
        let mut active: Vec<_> = (first..=span)
            .map(|index| {
                let index = index % count;
                let weight = self.control.weights.as_ref().map_or(1., |w| w[index]);
                (self.control.points[index], weight)
            })
            .collect();
        for level in 1..=degree {
            for j in (level..=degree).rev() {
                let index = first + j;
                let lo = self.knots[index];
                let hi = self.knots[index + degree - level + 1];
                let alpha = if (hi - lo).is_finite() {
                    (parameter - lo) / (hi - lo)
                } else {
                    (parameter * 0.5 - lo * 0.5) / (hi * 0.5 - lo * 0.5)
                };
                active[j] = blend(active[j - 1], active[j], alpha)?;
            }
        }
        Ok(active[degree].0)
    }
}

fn blend<const D: usize>(
    left: ([f64; D], f64),
    right: ([f64; D], f64),
    alpha: f64,
) -> Result<([f64; D], f64), SplineError> {
    if alpha == 0. {
        return Ok(left);
    }
    if alpha == 1. {
        return Ok(right);
    }
    // Normalize each pair before dividing to evaluate without multiplying a
    // potentially large coordinate by its homogeneous weight.
    let scale = left.1.max(right.1);
    let a = (1. - alpha) * (left.1 / scale);
    let b = alpha * (right.1 / scale);
    let sum = a + b;
    let weight = sum * scale;
    let point = std::array::from_fn(|i| (a / sum) * left.0[i] + (b / sum) * right.0[i]);
    if !weight.is_finite() || weight <= 0. || point.iter().any(|v| !v.is_finite()) {
        return Err(SplineError::EvaluationUnavailable);
    }
    Ok((point, weight))
}
