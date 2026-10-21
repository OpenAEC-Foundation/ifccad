//! Expand only validated native defaults for source-loss comparison.
//! Supplied values and foreign attributes are never replaced or removed.
use serde_json::{json, Map, Value};

fn fill(value: &mut Value, fields: &[(&str, Value)]) {
    if let Some(value) = value.as_object_mut() {
        for (key, default) in fields {
            value.entry(*key).or_insert_with(|| default.clone());
        }
    }
}

pub(super) fn expand_attributes(attrs: &mut Map<String, Value>) {
    for key in ["ifccad::geom::solid", "ifccad::geom::trace"] {
        if let Some(v) = attrs.get_mut(key) {
            fill(v, &[("thickness", json!(0.))]);
        }
    }
    for key in ["ifccad::entity", "ifccad::dimensionGraphicsPart"] {
        if let Some(entity) = attrs.get_mut(key) {
            fill(
                entity,
                &[("visible", json!(true)), ("appearance", json!({}))],
            );
            fill(
                &mut entity["appearance"],
                &[
                    ("color", json!({"mode":"ByLayer"})),
                    ("opacity", json!({"mode":"ByLayer"})),
                    ("linePattern", json!({"mode":"ByLayer"})),
                    ("lineWeight", json!({"mode":"ByLayer"})),
                ],
            );
        }
    }
    if let Some(part) = attrs.get_mut("ifccad::dimensionGraphicsPart") {
        fill(
            part,
            &[
                ("purpose", json!("display")),
                ("linePatternScale", json!(1.)),
            ],
        );
    }
    if let Some(layer) = attrs.get_mut("ifccad::layer") {
        fill(
            layer,
            &[
                ("visible", json!(true)),
                ("frozen", json!(false)),
                ("locked", json!(false)),
                ("plottable", json!(true)),
                ("frozenInNewViewports", json!(false)),
            ],
        );
        fill(&mut layer["appearance"], &[("opacity", json!(1.))]);
    }
    if let Some(instance) = attrs.get_mut("ifccad::blockInstance") {
        fill(
            &mut instance["transform"],
            &[("rotation", json!(0.)), ("scale", json!([1., 1., 1.]))],
        );
    }
    if let Some(line) = attrs.get_mut("ifccad::geom::lineSegment") {
        for field in ["start", "end"] {
            if let Some(point) = line.get_mut(field).and_then(Value::as_array_mut) {
                if point.len() == 2 {
                    point.push(json!(0.));
                }
            }
        }
    }
    for key in [
        "ifccad::geom::planarPolyline",
        "ifccad::geom::spatialPolyline",
    ] {
        if let Some(polyline) = attrs.get_mut(key) {
            fill(polyline, &[("closed", json!(false))]);
        }
    }
    for key in ["ifccad::text", "ifccad::mText"] {
        if let Some(text) = attrs.get_mut(key) {
            fill(
                text,
                &[
                    ("rotation", json!(0.)),
                    ("backward", json!(false)),
                    ("upsideDown", json!(false)),
                ],
            );
            if key == "ifccad::text" {
                fill(
                    text,
                    &[("obliqueAngle", json!(0.)), ("thickness", json!(0.))],
                );
            }
        }
    }
    if let Some(style) = attrs.get_mut("ifccad::textStyle") {
        fill(
            style,
            &[
                ("widthFactor", json!(1.)),
                ("obliqueAngle", json!(0.)),
                ("vertical", json!(false)),
                ("creationBackward", json!(false)),
                ("creationUpsideDown", json!(false)),
            ],
        );
    }
    if let Some(pattern) = attrs.get_mut("ifccad::linePattern") {
        if let Some(elements) = pattern.get_mut("pattern").and_then(Value::as_array_mut) {
            for element in elements {
                if let Some(marker) = element.get_mut("marker") {
                    fill(
                        marker,
                        &[
                            ("scale", json!(1.)),
                            ("offset", json!([0., 0.])),
                            ("rotation", json!({"mode":"relative","angle":0.})),
                        ],
                    );
                }
            }
        }
    }
    if let Some(hatch) = attrs.get_mut("ifccad::hatch") {
        if let Some(pattern) = hatch.get_mut("fill").filter(|v| v["kind"] == "linePattern") {
            fill(
                pattern,
                &[
                    ("origin", json!([0., 0.])),
                    ("rotation", json!(0.)),
                    ("scale", json!(1.)),
                ],
            );
        }
    }
}
