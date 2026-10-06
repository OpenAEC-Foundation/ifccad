//! Strict geometry payload fields, independent of shared geometric semantics.
use super::*;
use serde::Deserialize;
use serde_json::{json, Map, Value};

pub(super) const PAYLOADS: &[&str] = &[
    "ifccad::geom::point",
    "ifccad::geom::lineSegment",
    "ifccad::geom::circle",
    "ifccad::geom::arc",
    "ifccad::geom::ellipse",
    "ifccad::geom::ellipseArc",
    "ifccad::geom::planarPolyline",
    "ifccad::geom::spatialPolyline",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PointValue {}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LineValue {
    start: [f64; 3],
    end: [f64; 3],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CircleValue {
    radius: f64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ArcValue {
    radius: f64,
    start_parameter: f64,
    sweep_parameter: f64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EllipseValue {
    semi_major_radius: f64,
    semi_minor_radius: f64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EllipseArcValue {
    semi_major_radius: f64,
    semi_minor_radius: f64,
    start_parameter: f64,
    sweep_parameter: f64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PlanarValue {
    vertices: Vec<[f64; 2]>,
    closed: bool,
    #[serde(default)]
    line_pattern_generation: IfccadLinePatternGeneration,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PlanarWithBulges {
    vertices: Vec<[f64; 2]>,
    bulges: Vec<f64>,
    closed: bool,
    #[serde(default)]
    line_pattern_generation: IfccadLinePatternGeneration,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SpatialValue {
    vertices: Vec<[f64; 3]>,
    closed: bool,
    #[serde(default)]
    line_pattern_generation: IfccadLinePatternGeneration,
}
fn decode<T: for<'a> Deserialize<'a>>(value: &Value, path: &str) -> Result<T, IfccadReport> {
    serde_json::from_value(value.clone())
        .map_err(|e| IfccadReport::one(format!("{path}: invalid geometry payload: {e}")))
}
pub(super) fn decode_kind(
    attrs: &Map<String, Value>,
    key: &str,
    path: &str,
) -> Result<IfccadEntityKind, IfccadReport> {
    let v = &attrs[key];
    let frame = || {
        decode::<IfccadPlacement>(
            attrs
                .get("ifccad::geom::placement")
                .ok_or_else(|| IfccadReport::one(format!("{path}: missing geometry placement")))?,
            path,
        )
    };
    if matches!(
        key,
        "ifccad::geom::lineSegment" | "ifccad::geom::spatialPolyline"
    ) && attrs.contains_key("ifccad::geom::placement")
    {
        return Err(IfccadReport::one(format!(
            "{path}: direct XYZ geometry forbids placement"
        )));
    }
    Ok(match key {
        "ifccad::geom::point" => {
            let _: PointValue = decode(v, path)?;
            IfccadEntityKind::Point {
                placement: frame()?,
            }
        }
        "ifccad::geom::lineSegment" => {
            let p: LineValue = decode(v, path)?;
            IfccadEntityKind::LineSegment {
                start: p.start,
                end: p.end,
            }
        }
        "ifccad::geom::circle" => {
            let p: CircleValue = decode(v, path)?;
            IfccadEntityKind::Circle {
                radius: p.radius,
                placement: frame()?,
            }
        }
        "ifccad::geom::arc" => {
            let p: ArcValue = decode(v, path)?;
            IfccadEntityKind::Arc {
                radius: p.radius,
                start_parameter: p.start_parameter,
                sweep_parameter: p.sweep_parameter,
                placement: frame()?,
            }
        }
        "ifccad::geom::ellipse" => {
            let p: EllipseValue = decode(v, path)?;
            IfccadEntityKind::Ellipse {
                semi_major_radius: p.semi_major_radius,
                semi_minor_radius: p.semi_minor_radius,
                placement: frame()?,
            }
        }
        "ifccad::geom::ellipseArc" => {
            let p: EllipseArcValue = decode(v, path)?;
            IfccadEntityKind::EllipseArc {
                semi_major_radius: p.semi_major_radius,
                semi_minor_radius: p.semi_minor_radius,
                start_parameter: p.start_parameter,
                sweep_parameter: p.sweep_parameter,
                placement: frame()?,
            }
        }
        "ifccad::geom::planarPolyline" => {
            let (vertices, bulges, closed, line_pattern_generation) = if v.get("bulges").is_some() {
                let p: PlanarWithBulges = decode(v, path)?;
                (p.vertices, p.bulges, p.closed, p.line_pattern_generation)
            } else {
                let p: PlanarValue = decode(v, path)?;
                let b = vec![0.; p.vertices.len()];
                (p.vertices, b, p.closed, p.line_pattern_generation)
            };
            IfccadEntityKind::PlanarPolyline {
                vertices,
                bulges,
                closed,
                placement: frame()?,
                line_pattern_generation,
            }
        }
        "ifccad::geom::spatialPolyline" => {
            let p: SpatialValue = decode(v, path)?;
            IfccadEntityKind::SpatialPolyline {
                vertices: p.vertices,
                closed: p.closed,
                line_pattern_generation: p.line_pattern_generation,
            }
        }
        _ => {
            return Err(IfccadReport::one(format!(
                "{path}: unsupported geometry payload"
            )))
        }
    })
}
pub(super) fn encode_kind(kind: &IfccadEntityKind, attrs: &mut Map<String, Value>) -> bool {
    let (key, payload, placement) = match kind {
        IfccadEntityKind::Point { placement } => {
            ("ifccad::geom::point", json!({}), Some(placement))
        }
        IfccadEntityKind::LineSegment { start, end } => (
            "ifccad::geom::lineSegment",
            json!({"start":start,"end":end}),
            None,
        ),
        IfccadEntityKind::Circle { radius, placement } => (
            "ifccad::geom::circle",
            json!({"radius":radius}),
            Some(placement),
        ),
        IfccadEntityKind::Arc {
            radius,
            start_parameter,
            sweep_parameter,
            placement,
        } => (
            "ifccad::geom::arc",
            json!({"radius":radius,"startParameter":start_parameter,"sweepParameter":sweep_parameter}),
            Some(placement),
        ),
        IfccadEntityKind::Ellipse {
            semi_major_radius,
            semi_minor_radius,
            placement,
        } => (
            "ifccad::geom::ellipse",
            json!({"semiMajorRadius":semi_major_radius,"semiMinorRadius":semi_minor_radius}),
            Some(placement),
        ),
        IfccadEntityKind::EllipseArc {
            semi_major_radius,
            semi_minor_radius,
            start_parameter,
            sweep_parameter,
            placement,
        } => (
            "ifccad::geom::ellipseArc",
            json!({"semiMajorRadius":semi_major_radius,"semiMinorRadius":semi_minor_radius,"startParameter":start_parameter,"sweepParameter":sweep_parameter}),
            Some(placement),
        ),
        IfccadEntityKind::PlanarPolyline {
            vertices,
            bulges,
            closed,
            placement,
            line_pattern_generation,
        } => {
            let mut value = json!({"vertices":vertices,"closed":closed,"linePatternGeneration":line_pattern_generation});
            if bulges.iter().any(|v| *v != 0.) {
                value["bulges"] = json!(bulges);
            }
            ("ifccad::geom::planarPolyline", value, Some(placement))
        }
        IfccadEntityKind::SpatialPolyline {
            vertices,
            closed,
            line_pattern_generation,
        } => (
            "ifccad::geom::spatialPolyline",
            json!({"vertices":vertices,"closed":closed,"linePatternGeneration":line_pattern_generation}),
            None,
        ),
        _ => return false,
    };
    attrs.insert(key.into(), payload);
    if let Some(p) = placement {
        attrs.insert("ifccad::geom::placement".into(), json!(p));
    }
    true
}
