use super::adapters::Result;
use super::recipe::{Appearance, Drawing, Entity, Geometry, Layer, Property};
use ifccad::ifcdr::{IfcdrEntityRef, IfcdrLengthUnit};
use ifccad::package::{AppearanceProperty, DrawingLayoutKind, DrawingRef, LinePatternRef};
use ifccad_convert::cadcodec::{CadDocument, Color, EntityType, LineWeight, Transparency};
use serde_json::Value;

fn property<T, U>(source: AppearanceProperty<T>, explicit: impl FnOnce(T) -> U) -> Property<U> {
    match source {
        AppearanceProperty::ByLayer => Property::ByLayer,
        AppearanceProperty::ByBlock => Property::ByBlock,
        AppearanceProperty::Explicit(value) => Property::Explicit(explicit(value)),
    }
}
fn pattern(pattern: LinePatternRef<'_>) -> String {
    match pattern {
        LinePatternRef::Name(name) => name.to_ascii_lowercase(),
        LinePatternRef::IfcxIdentity(id) => format!("ifcx:{id}"),
    }
}

pub fn ifccad(drawing: DrawingRef<'_>) -> Result<(Drawing, Vec<u64>)> {
    let representation = drawing.representation();
    let resource = representation.resource();
    if resource.unit() != IfcdrLengthUnit::Millimetre {
        return Err("unit is not millimetre".into());
    }
    let layouts: Vec<_> = drawing.layouts().collect();
    if layouts.len() != 1 || resource.scopes().len() != 1 {
        return Err("expected one layout and scope".into());
    }
    if layouts[0].kind() != DrawingLayoutKind::Model {
        return Err("expected model layout".into());
    }
    let mut layers = Vec::new();
    for layer in representation.layers() {
        let a = layer.appearance().ok_or("layer appearance missing")?;
        layers.push(Layer {
            name: layer.name().into(),
            visible: layer.visible(),
            appearance: Appearance {
                color: Property::Explicit(a.color().rgb().components()),
                opacity: Property::Explicit(a.opacity()),
                pattern: Property::Explicit(pattern(a.line_pattern())),
                weight: Property::Explicit(a.line_weight()),
            },
        });
    }
    let mut entities = Vec::new();
    let mut ids = Vec::new();
    for entity in resource.entities(layouts[0].scope().id()) {
        let (geometry, layer, appearance, visible, id) = match entity {
            IfcdrEntityRef::Line(line) => (
                Geometry::Line {
                    start: [line.start().x(), line.start().y()],
                    end: [line.end().x(), line.end().y()],
                },
                line.layer_id(),
                line.appearance_id(),
                line.visible(),
                line.entity_id().get(),
            ),
            IfcdrEntityRef::Polyline(polyline) => (
                Geometry::Polyline {
                    points: polyline.points().map(|p| [p.x(), p.y()]).collect(),
                    closed: polyline.closed(),
                },
                polyline.layer_id(),
                polyline.appearance_id(),
                polyline.visible(),
                polyline.entity_id().get(),
            ),
        };
        let a = representation
            .appearance(appearance)
            .ok_or("entity appearance missing")?;
        entities.push(Entity {
            geometry,
            layer: representation
                .layer(layer)
                .ok_or("entity layer missing")?
                .name()
                .into(),
            visible,
            appearance: Appearance {
                color: property(a.color(), |c| c.rgb().components()),
                opacity: property(a.opacity(), |o| o),
                pattern: property(a.line_pattern(), pattern),
                weight: property(a.line_weight(), |w| w),
            },
        });
        ids.push(id);
    }
    layers.sort_by(|a, b| a.name.cmp(&b.name));
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
                if line.start.z == 0.
                    && line.end.z == 0.
                    && line.thickness == 0.
                    && line.normal.x == 0.
                    && line.normal.y == 0.
                    && line.normal.z == 1. =>
            {
                Geometry::Line {
                    start: [line.start.x, line.start.y],
                    end: [line.end.x, line.end.y],
                }
            }
            EntityType::LwPolyline(polyline)
                if polyline.elevation == 0.
                    && polyline.thickness == 0.
                    && polyline.constant_width == 0.
                    && polyline.normal.x == 0.
                    && polyline.normal.y == 0.
                    && polyline.normal.z == 1.
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
pub fn verify(expected: &Drawing, actual: &Drawing) -> Result<()> {
    if let Some(difference) = first_difference(
        &serde_json::to_value(expected)?,
        &serde_json::to_value(actual)?,
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
