use crate::ocdraw::read::{diagnostic, DrawingDiagnostic};
use crate::ocdraw::{CoordinateFrame3, Point3, Vector3};
use serde_json::Value;

pub(super) fn validate_polyline_geometry(
    _drawing: &Value,
    name: &str,
    stream: &Value,
    row: usize,
    offset: usize,
    count: usize,
    diagnostics: &mut Vec<DrawingDiagnostic>,
) {
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
                crate::ocdraw::Point2::new(
                    stream["x"][at].as_f64().unwrap_or(f64::NAN),
                    stream["y"][at].as_f64().unwrap_or(f64::NAN),
                )
            };
            let bulge = stream["bulge"][start].as_f64().unwrap_or(f64::NAN);
            let Some(segment) =
                crate::ocdraw::geometry::bulge_segment_bounds(point(start), point(end), bulge)
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
                    let Ok(point) = frame.try_to_scope_point(crate::ocdraw::Point2::new(x, y))
                    else {
                        diagnostics.push(diagnostic(
                            "ENTITY_GEOMETRY",
                            format!("/streams/{name}/vertexOffset/{row}"),
                            "polyline coordinate cannot be evaluated",
                        ));
                        return;
                    };
                    let _ = point;
                }
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
                    crate::ocdraw::geometry::circular_bounds(frame.components(), radius, sweep)
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
                        crate::ocdraw::geometry::elliptic_bounds(
                            frame.components(),
                            major,
                            minor,
                            sweep,
                        )
                    })
                }
            } else {
                let origin = frame.origin();
                Some(crate::ocdraw::Bounds3d {
                    min: origin,
                    max: origin,
                })
            };
            let Some(_) = enclosure else {
                diagnostics.push(diagnostic(
                    "ENTITY_GEOMETRY",
                    format!("/streams/{payload}/{row}"),
                    "invalid curve radius, sweep, or placement",
                ));
                continue;
            };
        }
    }
}
