use super::adapters::Result;
use super::recipe::{Appearance, Drawing, Entity, Geometry, Layer, Property};
use ocdraw::ocdraw::{AppearanceSelection, DrawingGeometry, ValidatedDrawing};
use ocdraw_convert::cadcodec::{CadDocument, Color, EntityType, LineWeight, Transparency};
use serde_json::Value;
fn property<T, U>(value: &AppearanceSelection<T>, convert: impl FnOnce(&T) -> U) -> Property<U> {
    match value {
        AppearanceSelection::ByLayer => Property::ByLayer,
        AppearanceSelection::ByBlock => Property::ByBlock,
        AppearanceSelection::Explicit(v) => Property::Explicit(convert(v)),
    }
}
pub fn ocdraw(drawing: &ValidatedDrawing) -> Result<(Drawing, Vec<u64>)> {
    if drawing.unit() != "mm" || drawing.scopes().len() != 1 || drawing.typed_layouts().len() != 1 {
        return Err("expected one millimetre model scope/layout".into());
    }
    let mut layers = drawing
        .typed_layers()
        .iter()
        .map(|l| Layer {
            name: l.name.clone(),
            visible: l.visible,
            appearance: Appearance {
                color: Property::Explicit(l.color.rgb),
                opacity: Property::Explicit(l.opacity),
                pattern: Property::Explicit(l.line_pattern.to_ascii_lowercase()),
                weight: Property::Explicit(l.line_weight),
            },
        })
        .collect::<Vec<_>>();
    layers.sort_by(|a, b| a.name.cmp(&b.name));
    let by_id = drawing
        .geometric_entities()
        .iter()
        .map(|e| (e.id(), e))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut entities = Vec::new();
    let mut ids = Vec::new();
    for id in &drawing.scopes()[0].entities {
        let e = by_id[id];
        let geometry = match e.geometry() {
            DrawingGeometry::Line { start, end } => Geometry::Line {
                start: *start,
                end: *end,
            },
            DrawingGeometry::PlanarPolyline {
                placement,
                vertices,
                closed,
            } => {
                if vertices.iter().any(|v| v[2] != 0.) {
                    return Err("bulge outside primitive corpus".into());
                }
                let o = placement.origin();
                let x = placement.x_axis();
                let y = placement.y_axis();
                Geometry::Polyline {
                    points: vertices.iter().map(|v| [v[0], v[1]]).collect(),
                    closed: *closed,
                    origin: [o.x(), o.y(), o.z()],
                    x_axis: [x.x(), x.y(), x.z()],
                    y_axis: [y.x(), y.y(), y.z()],
                }
            }
            _ => return Err("entity outside primitive corpus".into()),
        };
        let a = e.appearance();
        entities.push(Entity {
            geometry,
            layer: drawing
                .typed_layers()
                .iter()
                .find(|l| l.id == e.layer_id())
                .ok_or("layer missing")?
                .name
                .clone(),
            visible: e.visible(),
            appearance: Appearance {
                color: property(&a.color, |c| c.rgb),
                opacity: property(&a.opacity, |v| *v),
                pattern: property(&a.line_pattern, |s| s.to_ascii_lowercase()),
                weight: property(&a.line_weight, |v| *v),
            },
        });
        ids.push(*id);
    }
    Ok((
        Drawing {
            unit: "millimetre",
            layers,
            entities,
        },
        ids,
    ))
}

fn cad_color(color: Color) -> Result<Property<[u8; 3]>> {
    Ok(match color {
        Color::ByLayer => Property::ByLayer,
        Color::ByBlock => Property::ByBlock,
        Color::Rgb { r, g, b } => Property::Explicit([r, g, b]),
        // The recipe intentionally requires RGB, not an ACI approximation.
        other => return Err(format!("unexpected CAD color representation {other:?}").into()),
    })
}
fn cad_weight(weight: LineWeight) -> Property<f64> {
    match weight {
        LineWeight::ByLayer => Property::ByLayer,
        LineWeight::ByBlock => Property::ByBlock,
        LineWeight::Default => Property::Explicit(0.25),
        other => Property::Explicit(f64::from(other.value()) / 100.),
    }
}
fn cad_pattern(pattern: &str) -> Property<String> {
    match pattern.to_ascii_lowercase().as_str() {
        "" | "bylayer" => Property::ByLayer,
        "byblock" => Property::ByBlock,
        other => Property::Explicit(other.into()),
    }
}
fn cad_opacity(transparency: Transparency) -> Property<f64> {
    match transparency {
        Transparency::ByLayer => Property::ByLayer,
        Transparency::ByBlock => Property::ByBlock,
        Transparency::Explicit(transparency) => {
            Property::Explicit(1.0 - f64::from(transparency) / 255.)
        }
    }
}
pub fn cad(document: &CadDocument) -> Result<Drawing> {
    if document.header.insertion_units != 4 {
        return Err(format!(
            "unit: expected millimetre, got {}",
            document.header.insertion_units
        )
        .into());
    }
    let mut layers = Vec::new();
    for layer in document.layers.iter() {
        layers.push(Layer {
            name: layer.name.clone(),
            visible: layer.is_visible(),
            appearance: Appearance {
                color: cad_color(layer.color)?,
                opacity: cad_opacity(layer.transparency),
                pattern: cad_pattern(&layer.line_type),
                weight: cad_weight(layer.line_weight),
            },
        });
    }
    let mut entities = Vec::new();
    for (index, entity) in document.entities().enumerate() {
        let common = entity.common();
        let geometry = match entity {
            EntityType::Line(line)
                if line.thickness == 0.
                    && line.normal.x == 0.
                    && line.normal.y == 0.
                    && line.normal.z == 1. =>
            {
                Geometry::Line {
                    start: [line.start.x, line.start.y, line.start.z],
                    end: [line.end.x, line.end.y, line.end.z],
                }
            }
            EntityType::LwPolyline(polyline)
                if polyline.thickness == 0.
                    && polyline.constant_width == 0.
                    && (polyline.normal == ocdraw_convert::cadcodec::Vector3::UNIT_Z
                        || polyline.normal == ocdraw_convert::cadcodec::Vector3::UNIT_X)
                    && polyline
                        .vertices
                        .iter()
                        .all(|v| v.bulge == 0. && v.start_width == 0. && v.end_width == 0.) =>
            {
                Geometry::Polyline {
                    points: polyline
                        .vertices
                        .iter()
                        .map(|v| [v.location.x, v.location.y])
                        .collect(),
                    closed: polyline.is_closed,
                    origin: if polyline.normal.x == 1. {
                        [polyline.elevation, 0., 0.]
                    } else {
                        [0., 0., polyline.elevation]
                    },
                    x_axis: if polyline.normal.x == 1. {
                        [0., 1., 0.]
                    } else {
                        [1., 0., 0.]
                    },
                    y_axis: if polyline.normal.x == 1. {
                        [0., 0., 1.]
                    } else {
                        [0., 1., 0.]
                    },
                }
            }
            _ => {
                return Err(format!(
                    "entities/{index}/geometry: unexpected type or non-XY/width/bulge data"
                )
                .into())
            }
        };
        entities.push(Entity {
            geometry,
            layer: common.layer.clone(),
            visible: !common.invisible,
            appearance: Appearance {
                color: cad_color(common.color)?,
                opacity: cad_opacity(common.transparency),
                pattern: cad_pattern(&common.linetype),
                weight: cad_weight(common.line_weight),
            },
        });
    }
    layers.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Drawing {
        unit: "millimetre",
        layers,
        entities,
    })
}

/// Locate the first changed field rather than dumping thousands of entities.
pub fn first_difference(expected: &Value, actual: &Value, path: &str) -> Option<String> {
    if expected == actual {
        return None;
    }
    match (expected, actual) {
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return Some(format!(
                    "{path}/length: expected {}, got {}",
                    a.len(),
                    b.len()
                ));
            }
            a.iter()
                .zip(b)
                .enumerate()
                .find_map(|(i, (a, b))| first_difference(a, b, &format!("{path}/{i}")))
        }
        (Value::Object(a), Value::Object(b)) if a.keys().eq(b.keys()) => a
            .iter()
            .find_map(|(key, a)| first_difference(a, &b[key], &format!("{path}/{key}"))),
        _ => Some(format!("{path}: expected {expected}, got {actual}")),
    }
}
// These controlled recipes use coordinate-axis frames and bounded dyadic
// coordinates, so these additions are exact. Native parameterization is
// deliberately compared separately from the geometric exchange projection.
fn geometric_projection(drawing: &Drawing) -> Result<Value> {
    let mut value = serde_json::to_value(drawing)?;
    for (entity, original) in value["entities"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .zip(&drawing.entities)
    {
        if let Geometry::Polyline {
            points,
            closed,
            origin,
            x_axis,
            y_axis,
        } = &original.geometry
        {
            if !x_axis
                .iter()
                .chain(y_axis.iter())
                .all(|v| *v == 0. || *v == 1.)
            {
                return Err("non-axis recipe requires exact-rational projection".into());
            }
            let points: Vec<[f64; 3]> = points
                .iter()
                .map(|p| std::array::from_fn(|i| origin[i] + p[0] * x_axis[i] + p[1] * y_axis[i]))
                .collect();
            entity["geometry"] =
                serde_json::json!({"kind":"polyline","points":points,"closed":closed});
        }
    }
    Ok(value)
}
pub fn verify(expected: &Drawing, actual: &Drawing) -> Result<()> {
    if let Some(difference) = first_difference(
        &geometric_projection(expected)?,
        &geometric_projection(actual)?,
        "",
    ) {
        return Err(difference.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::{
        adapters,
        recipe::{generate, Case},
    };
    use super::*;
    #[test]
    fn direct_adapter_retains_all_mixed_properties_and_detects_changes() {
        let recipe = generate(&Case {
            id: "test".into(),
            family: "mixed".into(),
            count: 8,
        })
        .unwrap();
        let mut document = adapters::cad(&recipe).unwrap();
        verify(&recipe, &cad(&document).unwrap()).unwrap();
        document.layers.get_mut("Layer-1").unwrap().flags.off = true;
        let error = verify(&recipe, &cad(&document).unwrap())
            .unwrap_err()
            .to_string();
        assert!(error.contains("/layers/1/visible"), "{error}");
    }
    #[test]
    fn exact_coordinate_comparison_reports_index() {
        let expected = serde_json::json!({"entities":[{"start":[0.,1.]}]});
        let actual = serde_json::json!({"entities":[{"start":[0.,1.0000000000000002]}]});
        assert!(first_difference(&expected, &actual, "")
            .unwrap()
            .starts_with("/entities/0/start/1:"));
    }
}
