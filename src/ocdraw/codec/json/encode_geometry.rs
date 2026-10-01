use super::encode_color;
use crate::ocdraw::logical::{AppearanceSelection, DrawingEntityRecord, EntityGeometry};
use serde_json::{json, Map, Value};

fn mode<T>(selection: &AppearanceSelection<T>) -> &'static str {
    match selection {
        AppearanceSelection::ByLayer => "ByLayer",
        AppearanceSelection::ByBlock => "ByBlock",
        AppearanceSelection::Explicit(_) => "Explicit",
    }
}

fn appearance_value<T>(
    selection: &AppearanceSelection<T>,
    encode: impl FnOnce(&T) -> Value,
) -> Value {
    match selection {
        AppearanceSelection::Explicit(value) => encode(value),
        _ => Value::Null,
    }
}

fn placement_axes(origin: [f64; 3], x: [f64; 3], y: [f64; 3]) -> Value {
    json!({"origin":{"x":origin[0],"y":origin[1],"z":origin[2]},
        "X":{"x":x[0],"y":x[1],"z":x[2]},"Y":{"x":y[0],"y":y[1],"z":y[2]}})
}

pub(super) fn placement_frame(frame: crate::ocdraw::CoordinateFrame3) -> Value {
    placement_axes(
        frame.origin().components(),
        frame.x_axis().components(),
        frame.y_axis().components(),
    )
}

fn encode_geometry_fields(geometry: &EntityGeometry) -> Map<String, Value> {
    let mut fields = Map::new();
    match geometry {
        EntityGeometry::Line { start, end } => {
            for (name, value) in [
                ("x1", start[0]),
                ("y1", start[1]),
                ("z1", start[2]),
                ("x2", end[0]),
                ("y2", end[1]),
                ("z2", end[2]),
            ] {
                fields.insert(name.into(), json!(value));
            }
        }
        EntityGeometry::Point { placement } => {
            fields.insert("placement".into(), placement_frame(*placement));
        }
        EntityGeometry::Circle { placement, radius } => {
            fields.insert("placement".into(), placement_frame(*placement));
            fields.insert("radius".into(), json!(radius));
        }
        EntityGeometry::Arc {
            placement,
            radius,
            start_parameter,
            sweep_parameter,
        } => {
            fields.insert("placement".into(), placement_frame(*placement));
            fields.insert("radius".into(), json!(radius));
            fields.insert("startParameter".into(), json!(start_parameter));
            fields.insert("sweepParameter".into(), json!(sweep_parameter));
        }
        EntityGeometry::Ellipse {
            placement,
            semi_major_radius,
            semi_minor_radius,
            arc,
        } => {
            fields.insert("placement".into(), placement_frame(*placement));
            fields.insert("semiMajorRadius".into(), json!(semi_major_radius));
            fields.insert("semiMinorRadius".into(), json!(semi_minor_radius));
            if let Some((start, sweep)) = arc {
                fields.insert("startParameter".into(), json!(start));
                fields.insert("sweepParameter".into(), json!(sweep));
            }
        }
        EntityGeometry::PlanarPolyline {
            placement,
            vertices,
            closed,
            line_pattern_generation,
        } => {
            if *placement != crate::ocdraw::CoordinateFrame3::default() {
                fields.insert("placement".into(), placement_frame(*placement));
            }
            fields.insert("closed".into(), json!(closed));
            fields.insert(
                "linePatternGeneration".into(),
                json!(line_pattern_generation.token()),
            );
            for (axis, name) in ["x", "y", "bulge"].into_iter().enumerate() {
                fields.insert(
                    name.into(),
                    json!(vertices.iter().map(|v| v[axis]).collect::<Vec<_>>()),
                );
            }
        }
        EntityGeometry::SpatialPolyline {
            vertices,
            closed,
            line_pattern_generation,
        } => {
            fields.insert("closed".into(), json!(closed));
            fields.insert(
                "linePatternGeneration".into(),
                json!(line_pattern_generation.token()),
            );
            for (axis, name) in ["x", "y", "z"].into_iter().enumerate() {
                fields.insert(
                    name.into(),
                    json!(vertices.iter().map(|v| v[axis]).collect::<Vec<_>>()),
                );
            }
        }
        EntityGeometry::BlockInstance {
            definition_scope_id,
            transform,
        } => {
            fields.insert("definitionScopeId".into(), json!(definition_scope_id));
            let mut value = json!({"placement": placement_frame(transform.placement())});
            if transform.rotation() != 0.0 {
                value["rotation"] = json!(transform.rotation());
            }
            if transform.scale() != crate::ocdraw::Scale3::default() {
                let scale = transform.scale();
                value["scale"] = json!({"x":scale.x(), "y":scale.y(), "z":scale.z()});
            }
            fields.insert("transform".into(), value);
        }
    }
    fields
}

pub(crate) fn object_columns(rows: &[&DrawingEntityRecord]) -> Map<String, Value> {
    let mut columns = Map::new();
    columns.insert("count".into(), json!(rows.len()));
    let common = [
        ("entityId", rows.iter().map(|row| json!(row.id)).collect()),
        (
            "layerId",
            rows.iter().map(|row| json!(row.layer_id)).collect(),
        ),
        (
            "visible",
            rows.iter().map(|row| json!(row.visible)).collect(),
        ),
    ];
    for (name, values) in common {
        columns.insert(name.into(), Value::Array(values));
    }
    let fields = rows
        .iter()
        .map(|row| encode_geometry_fields(&row.geometry))
        .collect::<Vec<_>>();
    let names = fields
        .iter()
        .flat_map(|row| row.keys())
        .collect::<std::collections::BTreeSet<_>>();
    for name in names {
        columns.insert(
            name.clone(),
            Value::Array(
                fields
                    .iter()
                    .map(|row| {
                        row.get(name).cloned().unwrap_or_else(|| {
                            debug_assert_eq!(name, "placement");
                            placement_frame(crate::ocdraw::CoordinateFrame3::default())
                        })
                    })
                    .collect(),
            ),
        );
    }
    appearance_columns(
        &mut columns,
        &rows.iter().map(|row| &row.appearance).collect::<Vec<_>>(),
    );
    columns
}

pub(crate) fn polyline_columns(rows: &[&DrawingEntityRecord], kind: &str) -> Map<String, Value> {
    let mut columns = object_columns(rows);
    let pools = if kind == "planarPolyline" {
        &["x", "y", "bulge"][..]
    } else {
        &["x", "y", "z"][..]
    };
    let mut offsets = Vec::new();
    let mut counts = Vec::new();
    let mut next = 0usize;
    for row in rows {
        let count = match &row.geometry {
            EntityGeometry::PlanarPolyline { vertices, .. }
            | EntityGeometry::SpatialPolyline { vertices, .. } => vertices.len(),
            _ => unreachable!("only polylines use pooled vertices"),
        };
        offsets.push(next);
        counts.push(count);
        next += count;
    }
    columns.insert("vertexOffset".into(), json!(offsets));
    columns.insert("vertexCount".into(), json!(counts));
    for &pool in pools {
        let nested = columns.remove(pool).expect("writer pool");
        let flat = nested
            .as_array()
            .expect("writer rows")
            .iter()
            .flat_map(|row| row.as_array().expect("writer pool row").iter().cloned())
            .collect::<Vec<_>>();
        columns.insert(pool.into(), Value::Array(flat));
    }
    columns
}

pub(super) fn appearance_columns(
    columns: &mut Map<String, Value>,
    appearances: &[&crate::ocdraw::EntityAppearance],
) {
    columns.insert(
        "linePatternScale".into(),
        json!(appearances
            .iter()
            .map(|row| row.line_pattern_scale)
            .collect::<Vec<_>>()),
    );
    for property in ["color", "opacity", "linePattern", "lineWeight"] {
        let mode_name = format!("{property}Mode");
        let modes = appearances
            .iter()
            .map(|row| match property {
                "color" => mode(&row.color),
                "opacity" => mode(&row.opacity),
                "linePattern" => mode(&row.line_pattern),
                "lineWeight" => mode(&row.line_weight),
                _ => unreachable!(),
            })
            .map(|mode| json!(mode))
            .collect();
        columns.insert(mode_name, Value::Array(modes));
        let values = appearances
            .iter()
            .map(|row| match property {
                "color" => appearance_value(&row.color, encode_color),
                "opacity" => appearance_value(&row.opacity, |value| json!(value)),
                "linePattern" => appearance_value(&row.line_pattern, |value| json!(value)),
                "lineWeight" => appearance_value(&row.line_weight, |value| json!(value)),
                _ => unreachable!(),
            })
            .collect();
        columns.insert(
            if property == "linePattern" {
                "linePatternId"
            } else {
                property
            }
            .into(),
            Value::Array(values),
        );
    }
}
