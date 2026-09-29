use crate::drawing::read::{diagnostic, DrawingDiagnostic};
use crate::drawing::{CoordinateFrame3, Point3, Vector3};
use serde_json::Value;

pub(super) fn validate_polyline_geometry(
    drawing: &Value,
    name: &str,
    stream: &Value,
    row: usize,
    offset: usize,
    count: usize,
    diagnostics: &mut Vec<DrawingDiagnostic>,
) {
    let scope_id = stream["scopeId"][row].as_u64();
    let Some(scope) = drawing["scopes"]
        .as_array()
        .and_then(|scopes| scopes.iter().find(|scope| scope["id"].as_u64() == scope_id))
    else {
        return;
    };
    let Some(bounds) = scope["bounds"].as_object() else {
        return;
    };
    let mut points = Vec::new();
    if name == "planarPolylineStream" {
        let Some(frame) = row_placement(stream, row) else {
            diagnostics.push(diagnostic(
                "ENTITY_GEOMETRY",
                format!("/streams/{name}/placement/{row}"),
                "invalid polyline placement",
            ));
            return;
        };
        let segments = if stream["closed"][row].as_bool() == Some(true) {
            count
        } else {
            count - 1
        };
        for index in 0..segments {
            let start = offset + index;
            let end = offset + (index + 1) % count;
            let point = |at| {
                crate::drawing::Point2::new(
                    stream["x"][at].as_f64().unwrap_or(f64::NAN),
                    stream["y"][at].as_f64().unwrap_or(f64::NAN),
                )
            };
            let bulge = stream["bulge"][start].as_f64().unwrap_or(f64::NAN);
            let Some(segment) =
                crate::drawing::geometry::bulge_segment_bounds(point(start), point(end), bulge)
            else {
                diagnostics.push(diagnostic(
                    "ENTITY_GEOMETRY",
                    format!("/streams/{name}/bulge/{start}"),
                    "invalid polyline segment",
                ));
                return;
            };
            for x in [segment.min().x(), segment.max().x()] {
                for y in [segment.min().y(), segment.max().y()] {
                    let Ok(point) = frame.try_to_scope_point(crate::drawing::Point2::new(x, y))
                    else {
                        diagnostics.push(diagnostic(
                            "ENTITY_GEOMETRY",
                            format!("/streams/{name}/vertexOffset/{row}"),
                            "polyline coordinate cannot be evaluated",
                        ));
                        return;
                    };
                    points.push(point.components());
                }
            }
        }
    } else {
        for index in offset..offset + count {
            points.push([
                stream["x"][index].as_f64().unwrap_or(f64::NAN),
                stream["y"][index].as_f64().unwrap_or(f64::NAN),
                stream["z"][index].as_f64().unwrap_or(f64::NAN),
            ]);
        }
    }
    for point in points {
        for (axis, coordinate) in ["X", "Y", "Z"].into_iter().zip(point) {
            let min = bounds[&format!("min{axis}")]
                .as_f64()
                .expect("schema bounds");
            let max = bounds[&format!("max{axis}")]
                .as_f64()
                .expect("schema bounds");
            if !coordinate.is_finite() || coordinate < min || coordinate > max {
                diagnostics.push(diagnostic(
                    "SCOPE_BOUNDS",
                    format!("/streams/{name}/vertexOffset/{row}"),
                    "polyline lies outside declared scope bounds",
                ));
                return;
            }
        }
    }
}

fn coordinate(value: &Value, key: &str) -> Option<[f64; 3]> {
    let point = &value[key];
    Some([
        point["x"].as_f64()?,
        point["y"].as_f64()?,
        point["z"].as_f64()?,
    ])
}

fn row_placement(stream: &Value, row: usize) -> Option<CoordinateFrame3> {
    let value = stream["placement"].get(row);
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return Some(CoordinateFrame3::default());
    };
    let origin = coordinate(value, "origin")?;
    let x = coordinate(value, "X")?;
    let y = coordinate(value, "Y")?;
    CoordinateFrame3::try_new(
        Point3::new(origin[0], origin[1], origin[2]),
        Vector3::new(x[0], x[1], x[2]),
        Vector3::new(y[0], y[1], y[2]),
    )
    .ok()
}

pub(super) fn validate_placed_geometry(value: &Value, diagnostics: &mut Vec<DrawingDiagnostic>) {
    for (kind, payload) in [
        ("point", "pointStream"),
        ("circle", "circleStream"),
        ("arc", "arcStream"),
        ("ellipse", "ellipseStream"),
        ("ellipseArc", "ellipseArcStream"),
    ] {
        let stream = &value["streams"][payload];
        let Some(ids) = stream["entityId"].as_array() else {
            continue;
        };
        for row in 0..ids.len() {
            let Some(frame) = row_placement(stream, row) else {
                diagnostics.push(diagnostic(
                    "ENTITY_GEOMETRY",
                    format!("/streams/{payload}/placement/{row}"),
                    "invalid entity placement",
                ));
                continue;
            };
            let enclosure = if matches!(kind, "circle" | "arc") {
                let radius = stream["radius"].get(row).and_then(Value::as_f64);
                radius.and_then(|radius| {
                    let sweep = if kind == "arc" {
                        stream["startParameter"]
                            .get(row)
                            .and_then(Value::as_f64)
                            .zip(stream["sweepParameter"].get(row).and_then(Value::as_f64))
                    } else {
                        None
                    };
                    if kind == "arc" && sweep.is_none() {
                        return None;
                    }
                    crate::drawing::geometry::circular_bounds(frame.components(), radius, sweep)
                })
            } else if matches!(kind, "ellipse" | "ellipseArc") {
                let major = stream["semiMajorRadius"].get(row).and_then(Value::as_f64);
                let minor = stream["semiMinorRadius"].get(row).and_then(Value::as_f64);
                let sweep = if kind == "ellipseArc" {
                    stream["startParameter"]
                        .get(row)
                        .and_then(Value::as_f64)
                        .zip(stream["sweepParameter"].get(row).and_then(Value::as_f64))
                } else {
                    None
                };
                if kind == "ellipseArc" && sweep.is_none() {
                    None
                } else {
                    major.zip(minor).and_then(|(major, minor)| {
                        crate::drawing::geometry::elliptic_bounds(
                            frame.components(),
                            major,
                            minor,
                            sweep,
                        )
                    })
                }
            } else {
                let origin = frame.origin();
                Some(crate::drawing::Bounds3d {
                    min: origin,
                    max: origin,
                })
            };
            let Some(enclosure) = enclosure else {
                diagnostics.push(diagnostic(
                    "ENTITY_GEOMETRY",
                    format!("/streams/{payload}/{row}"),
                    "invalid curve radius, sweep, or placement",
                ));
                continue;
            };
            let scope_id = stream["scopeId"].get(row).and_then(Value::as_u64);
            let Some(bounds) = value["scopes"]
                .as_array()
                .and_then(|scopes| scopes.iter().find(|scope| scope["id"].as_u64() == scope_id))
                .and_then(|scope| scope["bounds"].as_object())
            else {
                continue;
            };
            for (index, axis) in ["X", "Y", "Z"].into_iter().enumerate() {
                let min = bounds[&format!("min{axis}")].as_f64();
                let max = bounds[&format!("max{axis}")].as_f64();
                let geometric_min = enclosure.min().components()[index];
                let geometric_max = enclosure.max().components()[index];
                if min.is_some_and(|min| geometric_min < min)
                    || max.is_some_and(|max| geometric_max > max)
                {
                    diagnostics.push(diagnostic(
                        "SCOPE_BOUNDS",
                        format!("/streams/{payload}/placement/{row}"),
                        "entity lies outside declared scope bounds",
                    ));
                }
            }
        }
    }
}
