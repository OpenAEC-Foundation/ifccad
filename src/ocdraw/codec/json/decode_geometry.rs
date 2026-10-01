//! Decode validated JSON columns into encoding-neutral geometric values.

use super::decode_appearance::appearance;
use crate::ocdraw::logical::{DrawingGeometricEntity, EntityGeometry};
use crate::ocdraw::{BlockTransform, CoordinateFrame3, Point3, Scale3, Vector3};
use serde_json::Value;
use std::collections::BTreeMap;

fn column<'a>(stream: &'a Value, field: &str, row: usize) -> Option<&'a Value> {
    stream.get(field)?.get(row)
}

fn number(stream: &Value, field: &str, row: usize) -> Option<f64> {
    column(stream, field, row)?.as_f64()
}

fn point(value: &Value) -> Option<[f64; 3]> {
    Some([
        value.get("x")?.as_f64()?,
        value.get("y")?.as_f64()?,
        value.get("z")?.as_f64()?,
    ])
}

fn placement(value: Option<&Value>) -> Option<CoordinateFrame3> {
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return Some(CoordinateFrame3::default());
    };
    let origin = point(value.get("origin")?)?;
    let x = point(value.get("X")?)?;
    let y = point(value.get("Y")?)?;
    CoordinateFrame3::try_new(
        Point3::new(origin[0], origin[1], origin[2]),
        Vector3::new(x[0], x[1], x[2]),
        Vector3::new(y[0], y[1], y[2]),
    )
    .ok()
}

fn geometry(stream: &Value, kind: &str, row: usize) -> Option<EntityGeometry> {
    Some(match kind {
        "line" => EntityGeometry::Line {
            start: [
                number(stream, "x1", row)?,
                number(stream, "y1", row)?,
                number(stream, "z1", row).unwrap_or(0.0),
            ],
            end: [
                number(stream, "x2", row)?,
                number(stream, "y2", row)?,
                number(stream, "z2", row).unwrap_or(0.0),
            ],
        },
        "point" => EntityGeometry::Point {
            placement: placement(column(stream, "placement", row))?,
        },
        "circle" => EntityGeometry::Circle {
            placement: placement(column(stream, "placement", row))?,
            radius: number(stream, "radius", row)?,
        },
        "arc" => EntityGeometry::Arc {
            placement: placement(column(stream, "placement", row))?,
            radius: number(stream, "radius", row)?,
            start_parameter: number(stream, "startParameter", row)?,
            sweep_parameter: number(stream, "sweepParameter", row)?,
        },
        "ellipse" | "ellipseArc" => EntityGeometry::Ellipse {
            placement: placement(column(stream, "placement", row))?,
            semi_major_radius: number(stream, "semiMajorRadius", row)?,
            semi_minor_radius: number(stream, "semiMinorRadius", row)?,
            arc: if kind == "ellipseArc" {
                Some((
                    number(stream, "startParameter", row)?,
                    number(stream, "sweepParameter", row)?,
                ))
            } else {
                None
            },
        },
        "planarPolyline" | "spatialPolyline" => {
            let offset = usize::try_from(column(stream, "vertexOffset", row)?.as_u64()?).ok()?;
            let count = usize::try_from(column(stream, "vertexCount", row)?.as_u64()?).ok()?;
            let end = offset.checked_add(count)?;
            let axis = if kind == "planarPolyline" {
                "bulge"
            } else {
                "z"
            };
            let vertices = (offset..end)
                .map(|at| {
                    Some([
                        stream.get("x")?.get(at)?.as_f64()?,
                        stream.get("y")?.get(at)?.as_f64()?,
                        stream.get(axis)?.get(at)?.as_f64()?,
                    ])
                })
                .collect::<Option<Vec<_>>>()?;
            let closed = column(stream, "closed", row)?.as_bool()?;
            if kind == "planarPolyline" {
                EntityGeometry::PlanarPolyline {
                    placement: placement(column(stream, "placement", row))?,
                    vertices,
                    closed,
                }
            } else {
                EntityGeometry::SpatialPolyline { vertices, closed }
            }
        }
        "blockInstance" => {
            let value = column(stream, "transform", row)?;
            let frame = placement(value.get("placement"))?;
            let rotation = value.get("rotation").and_then(Value::as_f64).unwrap_or(0.0);
            let scale = value
                .get("scale")
                .filter(|scale| !scale.is_null())
                .map(|scale| {
                    Some(Scale3::new(
                        scale.get("x")?.as_f64()?,
                        scale.get("y")?.as_f64()?,
                        scale.get("z")?.as_f64()?,
                    ))
                })
                .unwrap_or_else(|| Some(Scale3::default()))?;
            EntityGeometry::BlockInstance {
                definition_scope_id: u32::try_from(
                    column(stream, "definitionScopeId", row)?.as_u64()?,
                )
                .ok()?,
                transform: BlockTransform::try_new(frame, rotation, scale).ok()?,
            }
        }
        _ => return None,
    })
}

pub(crate) fn decode_geometric_entities(value: &Value) -> Option<Vec<DrawingGeometricEntity>> {
    let mut entities = Vec::new();
    for kind in [
        "line",
        "point",
        "circle",
        "arc",
        "ellipse",
        "ellipseArc",
        "planarPolyline",
        "spatialPolyline",
        "blockInstance",
    ] {
        let name = format!("{kind}Stream");
        let Some(stream) = value.get("streams")?.get(&name) else {
            continue;
        };
        let count = usize::try_from(stream.get("count")?.as_u64()?).ok()?;
        for row in 0..count {
            entities.push(DrawingGeometricEntity {
                id: column(stream, "entityId", row)?.as_u64()?,
                layer_id: u32::try_from(column(stream, "layerId", row)?.as_u64()?).ok()?,
                visible: column(stream, "visible", row)
                    .and_then(Value::as_bool)
                    .unwrap_or(true),
                appearance: appearance(stream, row)?,
                geometry: geometry(stream, kind, row)?,
            });
        }
    }
    let mut by_id = entities
        .into_iter()
        .map(|entity| (entity.id, entity))
        .collect::<BTreeMap<_, _>>();
    let mut ordered = Vec::with_capacity(by_id.len());
    if let Some(ids) = value["scopes"].as_array() {
        for scope in ids {
            for id in scope["entities"].as_array()? {
                if let Some(entity) = by_id.remove(&id.as_u64()?) {
                    ordered.push(entity);
                }
            }
        }
    }
    if !by_id.is_empty() {
        return None;
    }
    Some(ordered)
}
