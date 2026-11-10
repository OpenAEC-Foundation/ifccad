use super::geometry::{encode_line_point, LinePointValue};
use super::*;
use crate::geometry_kernel::{SplineControl, SplineKnots};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireSpline {
    topology: IfccadSplineTopology,
    definition: WireDefinition,
    normal: Option<[f64; 3]>,
    planar: Option<bool>,
    linear: Option<bool>,
    rational: Option<bool>,
    knot_tolerance: Option<f64>,
    control_tolerance: Option<f64>,
    authoring_mode: Option<IfccadSplineAuthoringMode>,
    control_frame_visible: Option<bool>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum WireDefinition {
    Control {
        basis: WireBasis<LinePointValue>,
        fit: Option<WireFit<LinePointValue, [f64; 3]>>,
    },
    FitOnly {
        degree: Option<u32>,
        fit: WireFit<LinePointValue, [f64; 3]>,
    },
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WireBasis<P> {
    degree: usize,
    knot_kind: KnotKind,
    knots: Vec<f64>,
    points: Vec<P>,
    #[serde(skip_serializing_if = "Option::is_none")]
    weights: Option<Vec<f64>>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum KnotKind {
    Full,
    PeriodicCycle,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WireFit<P, T> {
    points: Vec<P>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_tangent: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_tangent: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tolerance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameterization: Option<IfccadSplineParameterization>,
}

impl From<WireFit<LinePointValue, [f64; 3]>> for IfccadSplineFitData<3> {
    fn from(fit: WireFit<LinePointValue, [f64; 3]>) -> Self {
        Self {
            points: fit.points.into_iter().map(LinePointValue::xyz).collect(),
            start_tangent: fit.start_tangent,
            end_tangent: fit.end_tangent,
            tolerance: fit.tolerance,
            parameterization: fit.parameterization,
        }
    }
}

impl<const D: usize> From<WireBasis<[f64; D]>> for SplineControl<D> {
    fn from(basis: WireBasis<[f64; D]>) -> Self {
        Self {
            degree: basis.degree,
            knots: match basis.knot_kind {
                KnotKind::Full => SplineKnots::Full(basis.knots),
                KnotKind::PeriodicCycle => SplineKnots::PeriodicCycle(basis.knots),
            },
            points: basis.points,
            weights: basis.weights,
        }
    }
}

impl<const D: usize> From<SplineControl<D>> for WireBasis<[f64; D]> {
    fn from(basis: SplineControl<D>) -> Self {
        let (knot_kind, knots) = match basis.knots {
            SplineKnots::Full(v) => (KnotKind::Full, v),
            SplineKnots::PeriodicCycle(v) => (KnotKind::PeriodicCycle, v),
        };
        Self {
            degree: basis.degree,
            knot_kind,
            knots,
            points: basis.points,
            weights: basis.weights,
        }
    }
}

impl<const D: usize> From<WireFit<[f64; D], [f64; D]>> for IfccadSplineFitData<D> {
    fn from(fit: WireFit<[f64; D], [f64; D]>) -> Self {
        Self {
            points: fit.points,
            start_tangent: fit.start_tangent,
            end_tangent: fit.end_tangent,
            tolerance: fit.tolerance,
            parameterization: fit.parameterization,
        }
    }
}

impl<const D: usize> From<IfccadSplineFitData<D>> for WireFit<[f64; D], [f64; D]> {
    fn from(fit: IfccadSplineFitData<D>) -> Self {
        Self {
            points: fit.points,
            start_tangent: fit.start_tangent,
            end_tangent: fit.end_tangent,
            tolerance: fit.tolerance,
            parameterization: fit.parameterization,
        }
    }
}

pub(super) fn decode(value: &Value, path: &str) -> Result<IfccadSpline, IfccadReport> {
    let error = |message| crate::ifccad::diagnostics::failure("IFCCAD-SPLINE-001", path, message);
    let wire: WireSpline =
        serde_json::from_value(value.clone()).map_err(|e| error(e.to_string()))?;
    let definition = match wire.definition {
        WireDefinition::Control { basis, fit } => IfccadSplineDefinition::Control {
            basis: SplineControl {
                degree: basis.degree,
                knots: match basis.knot_kind {
                    KnotKind::Full => SplineKnots::Full(basis.knots),
                    KnotKind::PeriodicCycle => SplineKnots::PeriodicCycle(basis.knots),
                },
                points: basis.points.into_iter().map(LinePointValue::xyz).collect(),
                weights: basis.weights,
            },
            fit: fit.map(Into::into),
        },
        WireDefinition::FitOnly { degree, fit } => IfccadSplineDefinition::FitOnly {
            degree,
            fit: fit.into(),
        },
    };
    Ok(IfccadSpline {
        definition,
        topology: wire.topology,
        normal: wire.normal,
        planar: wire.planar,
        linear: wire.linear,
        rational: wire.rational,
        knot_tolerance: wire.knot_tolerance,
        control_tolerance: wire.control_tolerance,
        authoring_mode: wire.authoring_mode,
        control_frame_visible: wire.control_frame_visible,
    })
}

fn optional_fields(fields: [(&str, Value); 4], points: &[[f64; 3]]) -> Value {
    let mut out: Map<String, Value> = fields
        .into_iter()
        .filter(|(_, v)| !v.is_null())
        .map(|(key, value)| (key.into(), value))
        .collect();
    out.insert(
        "points".into(),
        Value::Array(points.iter().map(encode_line_point).collect()),
    );
    Value::Object(out)
}

fn encode_fit(fit: &IfccadSplineFitData<3>) -> Value {
    optional_fields(
        [
            ("startTangent", json!(fit.start_tangent)),
            ("endTangent", json!(fit.end_tangent)),
            ("tolerance", json!(fit.tolerance)),
            ("parameterization", json!(fit.parameterization)),
        ],
        &fit.points,
    )
}

pub(super) fn encode(spline: &IfccadSpline) -> Value {
    let definition = match &spline.definition {
        IfccadSplineDefinition::Control { basis, fit } => {
            let (kind, knots) = match &basis.knots {
                SplineKnots::Full(knots) => ("full", knots),
                SplineKnots::PeriodicCycle(knots) => ("periodicCycle", knots),
            };
            let basis = optional_fields(
                [
                    ("degree", json!(basis.degree)),
                    ("knotKind", json!(kind)),
                    ("knots", json!(knots)),
                    ("weights", json!(basis.weights)),
                ],
                &basis.points,
            );
            let mut definition = json!({"kind": "control", "basis": basis});
            if let Some(fit) = fit {
                definition["fit"] = encode_fit(fit);
            }
            definition
        }
        IfccadSplineDefinition::FitOnly { degree, fit } => {
            let mut definition = json!({"kind": "fitOnly", "fit": encode_fit(fit)});
            if let Some(degree) = degree {
                definition["degree"] = json!(degree);
            }
            definition
        }
    };
    let mut result = json!({
        "topology": spline.topology,
        "definition": definition,
        "normal": spline.normal,
        "planar": spline.planar,
        "linear": spline.linear,
        "rational": spline.rational,
        "knotTolerance": spline.knot_tolerance,
        "controlTolerance": spline.control_tolerance,
        "authoringMode": spline.authoring_mode,
        "controlFrameVisible": spline.control_frame_visible
    });
    result
        .as_object_mut()
        .unwrap()
        .retain(|_, value| !value.is_null());
    result
}
