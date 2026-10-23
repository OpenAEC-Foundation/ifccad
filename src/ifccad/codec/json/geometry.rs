//! Strict geometry payload fields, independent of shared geometric semantics.
use super::*;
use serde::Deserialize;
use serde_json::{json, Map, Value};

pub(super) const PAYLOADS: &[&str] = &[
    "ifccad::geom::solid",
    "ifccad::geom::trace",
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
struct FilledValue {
    corners: [[f64; 2]; 4],
    #[serde(default)]
    thickness: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PointValue {}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LineValue {
    start: LinePointValue,
    end: LinePointValue,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum LinePointValue {
    Xy([f64; 2]),
    Xyz([f64; 3]),
}
impl LinePointValue {
    fn xyz(self) -> [f64; 3] {
        match self {
            Self::Xy([x, y]) => [x, y, 0.0],
            Self::Xyz(point) => point,
        }
    }
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
    #[serde(default)]
    widths: IfccadPlanarPolylineWidths,
    vertices: Vec<[f64; 2]>,
    #[serde(default)]
    closed: bool,
    #[serde(default)]
    line_pattern_generation: IfccadLinePatternGeneration,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PlanarWithBulges {
    #[serde(default)]
    widths: IfccadPlanarPolylineWidths,
    vertices: Vec<[f64; 2]>,
    bulges: Vec<f64>,
    #[serde(default)]
    closed: bool,
    #[serde(default)]
    line_pattern_generation: IfccadLinePatternGeneration,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SpatialValue {
    vertices: Vec<[f64; 3]>,
    #[serde(default)]
    closed: bool,
    #[serde(default)]
    line_pattern_generation: IfccadLinePatternGeneration,
}
fn decode<T: for<'a> Deserialize<'a>>(value: &Value, path: &str) -> Result<T, IfccadReport> {
    serde_json::from_value(value.clone()).map_err(|e| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-GEOMETRY-001",
            path,
            format!("invalid geometry payload: {e}"),
        )
    })
}
pub(super) fn decode_kind(
    attrs: &Map<String, Value>,
    key: &str,
    path: &str,
) -> Result<IfccadEntityKind, IfccadReport> {
    let v = &attrs[key];
    super::supplemental::validate_value(key, v, path)?;
    let frame = || {
        if let Some(v) = attrs.get("ifccad::geom::placement") {
            super::supplemental::validate_value("ifccad::geom::placement", v, path)?;
        }
        decode::<IfccadPlacement>(
            attrs.get("ifccad::geom::placement").ok_or_else(|| {
                crate::ifccad::diagnostics::failure(
                    "IFCCAD-GEOMETRY-001",
                    &format!("{path}/ifccad::geom::placement"),
                    "missing geometry placement",
                )
            })?,
            path,
        )
    };
    if matches!(
        key,
        "ifccad::geom::lineSegment" | "ifccad::geom::spatialPolyline"
    ) && attrs.contains_key("ifccad::geom::placement")
    {
        return Err(crate::ifccad::diagnostics::failure(
            "IFCCAD-GEOMETRY-001",
            &format!("{path}/ifccad::geom::placement"),
            format!("{path}: direct XYZ geometry forbids placement"),
        ));
    }
    Ok(match key {
        "ifccad::geom::solid" | "ifccad::geom::trace" => {
            let v: FilledValue = decode(v, path)?;
            let q = IfccadFilledQuadrilateral {
                placement: frame()?,
                corners: v.corners,
                thickness: v.thickness,
            };
            if key == "ifccad::geom::trace" {
                IfccadEntityKind::Trace(q)
            } else {
                IfccadEntityKind::Solid(q)
            }
        }
        "ifccad::geom::point" => {
            let _: PointValue = decode(v, path)?;
            IfccadEntityKind::Point {
                placement: frame()?,
            }
        }
        "ifccad::geom::lineSegment" => {
            let p: LineValue = decode(v, path)?;
            IfccadEntityKind::LineSegment {
                start: p.start.xyz(),
                end: p.end.xyz(),
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
            let (vertices, bulges, closed, line_pattern_generation, widths) =
                if v.get("bulges").is_some() {
                    let p: PlanarWithBulges = decode(v, path)?;
                    (
                        p.vertices,
                        p.bulges,
                        p.closed,
                        p.line_pattern_generation,
                        p.widths,
                    )
                } else {
                    let p: PlanarValue = decode(v, path)?;
                    let b = vec![0.; p.vertices.len()];
                    (p.vertices, b, p.closed, p.line_pattern_generation, p.widths)
                };
            IfccadEntityKind::PlanarPolyline {
                widths,
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
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-GEOMETRY-001",
                &format!("{path}/{key}"),
                format!("{path}: unsupported geometry payload"),
            ))
        }
    })
}
pub(super) fn encode_kind(kind: &IfccadEntityKind, attrs: &mut Map<String, Value>) -> bool {
    let (key, payload, placement) = match kind {
        IfccadEntityKind::Solid(v) | IfccadEntityKind::Trace(v) => {
            let mut payload = json!({"corners":v.corners});
            if v.thickness.to_bits() != 0 {
                payload["thickness"] = json!(v.thickness);
            }
            (
                if matches!(kind, IfccadEntityKind::Trace(_)) {
                    "ifccad::geom::trace"
                } else {
                    "ifccad::geom::solid"
                },
                payload,
                Some(&v.placement),
            )
        }
        IfccadEntityKind::Point { placement } => {
            ("ifccad::geom::point", json!({}), Some(placement))
        }
        IfccadEntityKind::LineSegment { start, end } => (
            "ifccad::geom::lineSegment",
            json!({"start":encode_line_point(start),"end":encode_line_point(end)}),
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
            widths,
            vertices,
            bulges,
            closed,
            placement,
            line_pattern_generation,
        } => {
            let mut value = json!({"vertices":vertices,"closed":closed,"linePatternGeneration":line_pattern_generation});
            if !widths.is_intrinsic_default() {
                value["widths"] = json!(widths);
            }
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
    let mut payload = payload;
    if matches!(
        kind,
        IfccadEntityKind::PlanarPolyline { .. } | IfccadEntityKind::SpatialPolyline { .. }
    ) {
        super::defaults::omit_bool(&mut payload, "closed", false);
        if payload["linePatternGeneration"] == "perSegment" {
            payload
                .as_object_mut()
                .expect("polyline")
                .remove("linePatternGeneration");
        }
    }
    attrs.insert(key.into(), payload);
    if let Some(p) = placement {
        attrs.insert("ifccad::geom::placement".into(), json!(p));
    }
    true
}
fn encode_line_point(point: &[f64; 3]) -> Value {
    if super::defaults::is_positive_zero(point[2]) {
        json!(&point[..2])
    } else {
        json!(point)
    }
}
