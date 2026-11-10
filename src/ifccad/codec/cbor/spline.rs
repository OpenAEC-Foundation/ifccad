use super::geometry::{decode_line_point, encode_line_point};
use super::*;
use crate::geometry_kernel::{SplineControl, SplineKnots};

record!(IfccadSpline;
    0 => definition: IfccadSplineDefinition = None,
    1 => topology: IfccadSplineTopology = None,
    2 => normal: Option<[f64; 3]> = Some(None),
    3 => planar: Option<bool> = Some(None),
    4 => linear: Option<bool> = Some(None),
    5 => rational: Option<bool> = Some(None),
    6 => knot_tolerance: Option<f64> = Some(None),
    7 => control_tolerance: Option<f64> = Some(None),
    8 => authoring_mode: Option<IfccadSplineAuthoringMode> = Some(None),
    9 => control_frame_visible: Option<bool> = Some(None),
);
unit_enum!(IfccadSplineTopology; 0 => Open, 1 => Closed, 2 => Periodic);
unit_enum!(IfccadSplineAuthoringMode; 0 => Control, 1 => Fit);
unit_enum!(IfccadSplineParameterization; 0 => Chord, 1 => Centripetal, 2 => Uniform, 3 => Custom);

record!(SplineControl<2>;
    0 => degree: usize = None,
    1 => knots: SplineKnots = None,
    2 => points: Vec<[f64; 2]> = None,
    3 => weights: Option<Vec<f64>> = Some(None),
);
record!(IfccadSplineFitData<2>;
    0 => points: Vec<[f64; 2]> = None,
    1 => start_tangent: Option<[f64; 2]> = Some(None),
    2 => end_tangent: Option<[f64; 2]> = Some(None),
    3 => tolerance: Option<f64> = Some(None),
    4 => parameterization: Option<IfccadSplineParameterization> = Some(None),
);

#[derive(Clone, Copy)]
struct SplinePoint([f64; 3]);

impl CborValue for SplinePoint {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        encode_line_point(&self.0, w)
    }

    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        decode_line_point(r).map(Self)
    }

    fn exact(&self, other: &Self) -> bool {
        self.0.exact(&other.0)
    }
}

impl CborValue for SplineKnots {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        let (kind, knots) = match self {
            Self::Full(v) => (0, v),
            Self::PeriodicCycle(v) => (1, v),
        };
        w.map(2)?;
        w.u64(0)?;
        w.u64(kind)?;
        w.u64(1)?;
        knots.encode(w)
    }

    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "knots")])?;
        let kind: u64 = f.take(0, None)?;
        let knots = f.take(1, None)?;
        f.finish()?;
        match kind {
            0 => Ok(Self::Full(knots)),
            1 => Ok(Self::PeriodicCycle(knots)),
            _ => Err(r.error("unknown spline knot representation")),
        }
    }

    fn exact(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Full(a), Self::Full(b)) | (Self::PeriodicCycle(a), Self::PeriodicCycle(b)) => {
                a.exact(b)
            }
            _ => false,
        }
    }
}

impl CborValue for SplineControl<3> {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.map(3 + u64::from(self.weights.is_some()))?;
        w.u64(0)?;
        w.u64(self.degree as u64)?;
        w.u64(1)?;
        self.knots.encode(w)?;
        w.u64(2)?;
        w.array(self.points.len() as u64)?;
        for point in &self.points {
            encode_line_point(point, w)?;
        }
        if let Some(weights) = &self.weights {
            w.u64(3)?;
            weights.encode(w)?;
        }
        Ok(())
    }

    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(
            r,
            &[(0, "degree"), (1, "knots"), (2, "points"), (3, "weights")],
        )?;
        let degree: u64 = f.take(0, None)?;
        let knots = f.take(1, None)?;
        let points: Vec<SplinePoint> = f.take(2, None)?;
        let weights = f.take(3, Some(None))?;
        f.finish()?;
        Ok(Self {
            degree: degree
                .try_into()
                .map_err(|_| r.error("spline degree out of range"))?,
            knots,
            points: points.into_iter().map(|p| p.0).collect(),
            weights,
        })
    }

    fn exact(&self, other: &Self) -> bool {
        self.degree == other.degree
            && self.knots.exact(&other.knots)
            && self.points.exact(&other.points)
            && self.weights.exact(&other.weights)
    }
}

impl CborValue for IfccadSplineFitData<3> {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        let mask = [
            self.start_tangent.is_some(),
            self.end_tangent.is_some(),
            self.tolerance.is_some(),
            self.parameterization.is_some(),
        ];
        w.map(1 + mask.into_iter().map(u64::from).sum::<u64>())?;
        w.u64(0)?;
        w.array(self.points.len() as u64)?;
        for point in &self.points {
            encode_line_point(point, w)?;
        }
        if let Some(value) = self.start_tangent {
            w.u64(1)?;
            value.encode(w)?;
        }
        if let Some(value) = self.end_tangent {
            w.u64(2)?;
            value.encode(w)?;
        }
        if let Some(value) = self.tolerance {
            w.u64(3)?;
            value.encode(w)?;
        }
        if let Some(value) = self.parameterization {
            w.u64(4)?;
            value.encode(w)?;
        }
        Ok(())
    }

    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(
            r,
            &[
                (0, "points"),
                (1, "start_tangent"),
                (2, "end_tangent"),
                (3, "tolerance"),
                (4, "parameterization"),
            ],
        )?;
        let points: Vec<SplinePoint> = f.take(0, None)?;
        let value = Self {
            points: points.into_iter().map(|p| p.0).collect(),
            start_tangent: f.take(1, Some(None))?,
            end_tangent: f.take(2, Some(None))?,
            tolerance: f.take(3, Some(None))?,
            parameterization: f.take(4, Some(None))?,
        };
        f.finish()?;
        Ok(value)
    }

    fn exact(&self, other: &Self) -> bool {
        self.points.exact(&other.points)
            && self.start_tangent.exact(&other.start_tangent)
            && self.end_tangent.exact(&other.end_tangent)
            && self.tolerance.exact(&other.tolerance)
            && self.parameterization.exact(&other.parameterization)
    }
}

impl CborValue for IfccadSplineDefinition {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.map(2)?;
        w.u64(0)?;
        match self {
            Self::Control { basis, fit } => {
                w.u64(0)?;
                w.u64(1)?;
                w.map(1 + u64::from(fit.is_some()))?;
                w.u64(0)?;
                basis.encode(w)?;
                if let Some(fit) = fit {
                    w.u64(1)?;
                    fit.encode(w)?;
                }
            }
            Self::FitOnly { degree, fit } => {
                w.u64(1)?;
                w.u64(1)?;
                w.map(1 + u64::from(degree.is_some()))?;
                if let Some(degree) = degree {
                    w.u64(0)?;
                    degree.encode(w)?;
                }
                w.u64(1)?;
                fit.encode(w)?;
            }
        }
        Ok(())
    }

    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let kind: u64 = f.take(0, None)?;
        let body = f.span(1)?;
        f.finish()?;
        match kind {
            0 => {
                let mut b = RecordFields::new(&body, &[(0, "basis"), (1, "fit")])?;
                let basis = b.take(0, None)?;
                let fit = b.take(1, Some(None))?;
                b.finish()?;
                Ok(Self::Control { basis, fit })
            }
            1 => {
                let mut b = RecordFields::new(&body, &[(0, "degree"), (1, "fit")])?;
                let degree = b.take(0, Some(None))?;
                let fit = b.take(1, None)?;
                b.finish()?;
                Ok(Self::FitOnly { degree, fit })
            }
            _ => Err(r.error("unknown spline definition")),
        }
    }

    fn exact(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Control { basis: a, fit: af }, Self::Control { basis: b, fit: bf }) => {
                a.exact(b) && af.exact(bf)
            }
            (Self::FitOnly { degree: a, fit: af }, Self::FitOnly { degree: b, fit: bf }) => {
                a.exact(b) && af.exact(bf)
            }
            _ => false,
        }
    }
}
