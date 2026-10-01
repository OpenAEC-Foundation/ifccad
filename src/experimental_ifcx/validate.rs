use super::*;
use crate::ocdraw::{BlockTransform, CoordinateFrame3, Point3, Scale3, Vector3};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DrawingValue {
    profile_version: String,
    length_unit: String,
}
#[derive(Deserialize)]
struct LayoutValue {
    kind: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LayerValue {
    name: String,
    appearance: IfcxCadLayerAppearance,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DefinitionValue {
    name: String,
    base_point: [f64; 3],
    insertion_unit: String,
}
#[derive(Deserialize)]
struct EntityValue {
    layer: String,
    appearance: IfcxCadEntityAppearance,
}
#[derive(Deserialize)]
struct LineValue {
    start: [f64; 3],
    end: [f64; 3],
}
#[derive(Deserialize)]
struct PolylineValue {
    vertices: Vec<[f64; 2]>,
    closed: bool,
}
#[derive(Deserialize)]
struct CircleValue {
    radius: f64,
}
#[derive(Deserialize)]
struct InstanceValue {
    definition: String,
    transform: IfcxCadBlockTransform,
}

fn problem(message: impl Into<String>) -> IfcxCadReport {
    IfcxCadReport::one(message)
}
fn attr<'a>(node: &'a Value, key: &str) -> Option<&'a Value> {
    node.get("attributes")?.get(key)
}
fn required<T: for<'de> Deserialize<'de>>(node: &Value, key: &str) -> Result<T, IfcxCadReport> {
    let path = node["path"].as_str().unwrap_or("<unknown>");
    let value = attr(node, key).ok_or_else(|| problem(format!("{path} missing {key}")))?;
    serde_json::from_value(value.clone()).map_err(|e| problem(format!("{path} invalid {key}: {e}")))
}
fn numbered(path: &str, prefix: &str) -> Result<u64, IfcxCadReport> {
    let suffix = path
        .strip_prefix(prefix)
        .and_then(|v| v.strip_suffix('>'))
        .ok_or_else(|| {
            problem(format!(
                "invalid CAD path {path}; expected {prefix}<number>>"
            ))
        })?;
    let number = suffix
        .parse::<u64>()
        .map_err(|_| problem(format!("invalid CAD path {path}")))?;
    if number.to_string() != suffix {
        return Err(problem(format!("noncanonical CAD path {path}")));
    }
    Ok(number)
}
fn drawing_prefix(id: u64) -> String {
    format!("</cad/d{id}")
}
fn node_map(raw: &Value) -> Result<BTreeMap<String, &Value>, IfcxCadReport> {
    let mut nodes = BTreeMap::new();
    for node in raw["data"]
        .as_array()
        .ok_or_else(|| problem("IFCX data must be an array"))?
    {
        let path = node
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| problem("IFCX node needs string path"))?;
        if nodes.insert(path.to_owned(), node).is_some() {
            return Err(problem(format!("duplicate path {path}")));
        }
    }
    Ok(nodes)
}
fn children(node: &Value) -> Result<Vec<String>, IfcxCadReport> {
    let path = node["path"].as_str().unwrap_or("<unknown>");
    let mut ordered = BTreeMap::<usize, String>::new();
    if let Some(children) = node.get("children") {
        for (key, value) in children
            .as_object()
            .ok_or_else(|| problem(format!("{path} children must be a map")))?
        {
            if key.chars().all(|c| c.is_ascii_digit()) && !key.is_empty() {
                let index = key
                    .parse::<usize>()
                    .map_err(|_| problem(format!("{path} child key out of range")))?;
                if index.to_string() != *key {
                    return Err(problem(format!("{path} noncanonical child key {key}")));
                }
                let target = value
                    .as_str()
                    .ok_or_else(|| problem(format!("{path} child {key} must reference a path")))?;
                ordered.insert(index, target.to_owned());
            }
        }
    }
    for (expected, actual) in ordered.keys().enumerate() {
        if expected != *actual {
            return Err(problem(format!("{path} numbered children have a gap")));
        }
    }
    Ok(ordered.into_values().collect())
}
fn finite3(value: [f64; 3], context: &str) -> Result<(), IfcxCadReport> {
    if value.iter().all(|n| n.is_finite()) {
        Ok(())
    } else {
        Err(problem(format!("{context} has non-finite coordinate")))
    }
}
fn placement(value: &IfcxCadPlacement, context: &str) -> Result<CoordinateFrame3, IfcxCadReport> {
    finite3(value.origin, context)?;
    finite3(value.x_axis, context)?;
    finite3(value.y_axis, context)?;
    let p = |v: [f64; 3]| Point3::new(v[0], v[1], v[2]);
    let v = |v: [f64; 3]| Vector3::new(v[0], v[1], v[2]);
    CoordinateFrame3::try_new(p(value.origin), v(value.x_axis), v(value.y_axis))
        .map_err(|e| problem(format!("{context} invalid placement: {e}")))
}
fn unit(token: &str) -> bool {
    matches!(
        token,
        "unitless"
            | "mm"
            | "cm"
            | "m"
            | "km"
            | "in"
            | "ft"
            | "mi"
            | "microin"
            | "mil"
            | "yd"
            | "angstrom"
            | "nm"
            | "um"
            | "dm"
            | "dam"
            | "hm"
            | "Gm"
            | "au"
            | "ly"
            | "pc"
            | "usSurveyFoot"
            | "usSurveyInch"
            | "usSurveyYard"
            | "usSurveyMile"
    )
}
fn color(value: &str) -> bool {
    value.len() == 7 && value.starts_with('#') && value[1..].bytes().all(|b| b.is_ascii_hexdigit())
}
fn appearance(layer: &IfcxCadLayerAppearance, context: &str) -> Result<(), IfcxCadReport> {
    if !color(&layer.color)
        || !layer.opacity.is_finite()
        || !(0.0..=1.0).contains(&layer.opacity)
        || layer.line_pattern != "Continuous"
        || !layer.line_weight.is_finite()
        || layer.line_weight < 0.0
    {
        return Err(problem(format!("{context} invalid layer appearance")));
    }
    Ok(())
}
fn entity_appearance(a: &IfcxCadEntityAppearance, context: &str) -> Result<(), IfcxCadReport> {
    if matches!(&a.color, IfcxCadMode::Explicit(v) if !color(v))
        || matches!(&a.opacity, IfcxCadMode::Explicit(v) if !v.is_finite() || !(0.0..=1.0).contains(v))
        || matches!(&a.line_pattern, IfcxCadMode::Explicit(v) if v != "Continuous")
        || matches!(&a.line_weight, IfcxCadMode::Explicit(v) if !v.is_finite() || *v < 0.0)
    {
        return Err(problem(format!("{context} invalid entity appearance")));
    }
    Ok(())
}
fn entity(
    node: &Value,
    prefix: &str,
    layer_paths: &BTreeMap<String, u64>,
    block_paths: &BTreeMap<String, u64>,
) -> Result<IfcxCadEntity, IfcxCadReport> {
    let path = node["path"].as_str().unwrap();
    let id = numbered(path, &format!("{prefix}/e"))?;
    let modes = attr(node, "ifccad::entity")
        .and_then(|value| value.get("appearance"))
        .and_then(Value::as_object)
        .ok_or_else(|| problem(format!("{path} missing entity appearance")))?;
    for property in ["color", "opacity", "linePattern", "lineWeight"] {
        let fields = modes
            .get(property)
            .and_then(Value::as_object)
            .ok_or_else(|| problem(format!("{path} missing {property} appearance mode")))?;
        let mode = fields
            .get("mode")
            .and_then(Value::as_str)
            .ok_or_else(|| problem(format!("{path} missing {property} mode")))?;
        let valid = match mode {
            "Explicit" => fields.len() == 2 && fields.contains_key("value"),
            "ByLayer" | "ByBlock" => fields.len() == 1,
            _ => false,
        };
        if !valid {
            return Err(problem(format!(
                "{path} invalid {property} mode/value combination"
            )));
        }
    }
    let value: EntityValue = required(node, "ifccad::entity")?;
    let layer_id = *layer_paths
        .get(&value.layer)
        .ok_or_else(|| problem(format!("{path} unresolved layer {}", value.layer)))?;
    entity_appearance(&value.appearance, path)?;
    let attrs = node
        .get("attributes")
        .and_then(Value::as_object)
        .ok_or_else(|| problem(format!("{path} missing attributes")))?;
    let payloads = [
        "ifccad::geom::lineSegment",
        "ifccad::geom::planarPolyline",
        "ifccad::geom::circle",
        "ifccad::blockInstance",
    ];
    let present: Vec<_> = payloads
        .iter()
        .filter(|key| attrs.contains_key(**key))
        .collect();
    if present.len() != 1 {
        return Err(problem(format!(
            "{path} needs exactly one drawable payload"
        )));
    }
    if attrs.keys().any(|key| {
        key.starts_with("ifccad::geom::")
            && key != "ifccad::geom::placement"
            && !payloads.contains(&key.as_str())
    }) {
        return Err(problem(format!("{path} unsupported CAD geometry")));
    }
    let kind = match *present[0] {
        "ifccad::geom::lineSegment" => {
            let line: LineValue = required(node, present[0])?;
            finite3(line.start, path)?;
            finite3(line.end, path)?;
            IfcxCadEntityKind::LineSegment {
                start: line.start,
                end: line.end,
            }
        }
        "ifccad::geom::planarPolyline" => {
            let poly: PolylineValue = required(node, present[0])?;
            if poly.vertices.len() < 2 || poly.vertices.iter().flatten().any(|n| !n.is_finite()) {
                return Err(problem(format!("{path} invalid polyline vertices")));
            }
            let frame: IfcxCadPlacement = required(node, "ifccad::geom::placement")?;
            placement(&frame, path)?;
            IfcxCadEntityKind::PlanarPolyline {
                vertices: poly.vertices,
                closed: poly.closed,
                placement: frame,
            }
        }
        "ifccad::geom::circle" => {
            let circle: CircleValue = required(node, present[0])?;
            if !circle.radius.is_finite() || circle.radius <= 0.0 {
                return Err(problem(format!("{path} invalid circle radius")));
            }
            let frame: IfcxCadPlacement = required(node, "ifccad::geom::placement")?;
            placement(&frame, path)?;
            IfcxCadEntityKind::Circle {
                radius: circle.radius,
                placement: frame,
            }
        }
        _ => {
            let instance: InstanceValue = required(node, present[0])?;
            let definition_id = *block_paths.get(&instance.definition).ok_or_else(|| {
                problem(format!(
                    "{path} unresolved block definition {}",
                    instance.definition
                ))
            })?;
            let frame = placement(&instance.transform.placement, path)?;
            BlockTransform::try_new(
                frame,
                instance.transform.rotation,
                Scale3::new(
                    instance.transform.scale[0],
                    instance.transform.scale[1],
                    instance.transform.scale[2],
                ),
            )
            .map_err(|e| problem(format!("{path} invalid block transform: {e}")))?;
            IfcxCadEntityKind::BlockInstance {
                definition_id,
                transform: instance.transform,
            }
        }
    };
    Ok(IfcxCadEntity {
        id,
        layer_id,
        appearance: value.appearance,
        kind,
    })
}
fn cycle(blocks: &[IfcxCadBlockDefinition]) -> bool {
    fn visit(
        id: u64,
        graph: &BTreeMap<u64, Vec<u64>>,
        active: &mut BTreeSet<u64>,
        done: &mut BTreeSet<u64>,
    ) -> bool {
        if done.contains(&id) {
            return false;
        }
        if !active.insert(id) {
            return true;
        }
        if graph.get(&id).is_some_and(|targets| {
            targets
                .iter()
                .any(|target| visit(*target, graph, active, done))
        }) {
            return true;
        }
        active.remove(&id);
        done.insert(id);
        false
    }
    let graph: BTreeMap<_, _> = blocks
        .iter()
        .map(|b| {
            (
                b.id,
                b.entities
                    .iter()
                    .filter_map(|e| match e.kind {
                        IfcxCadEntityKind::BlockInstance { definition_id, .. } => {
                            Some(definition_id)
                        }
                        _ => None,
                    })
                    .collect(),
            )
        })
        .collect();
    let mut active = BTreeSet::new();
    let mut done = BTreeSet::new();
    graph
        .keys()
        .any(|id| visit(*id, &graph, &mut active, &mut done))
}

pub(super) fn validate(raw: Value) -> Result<ValidatedIfcxCad, IfcxCadReport> {
    let imports = raw
        .get("imports")
        .and_then(Value::as_array)
        .ok_or_else(|| problem("IFCX imports must be an array"))?;
    let imported_profile = imports
        .iter()
        .any(|item| item.get("uri").and_then(Value::as_str) == Some(PROFILE_URI));
    let schemas = raw
        .get("schemas")
        .and_then(Value::as_object)
        .ok_or_else(|| problem("IFCX schemas must be an object"))?;
    let profile_module: Value = serde_json::from_str(include_str!(
        "../../schemas/ifcx-native-cad/experimental-profile-0.1.0.ifcx"
    ))
    .map_err(|e| problem(format!("invalid built-in CAD schema module: {e}")))?;
    let known = profile_module["schemas"]
        .as_object()
        .expect("built-in schema module has schemas");
    for node in raw["data"]
        .as_array()
        .ok_or_else(|| problem("IFCX data must be an array"))?
    {
        if let Some(attributes) = node.get("attributes").and_then(Value::as_object) {
            for key in attributes.keys().filter(|key| key.starts_with("ifccad::")) {
                if let Some(expected) = known.get(key) {
                    if schemas.get(key).is_some_and(|actual| actual != expected)
                        || (schemas.get(key).is_none() && !imported_profile)
                    {
                        return Err(problem(format!("missing or changed CAD schema {key}")));
                    }
                }
            }
        }
    }
    let header = raw
        .get("header")
        .ok_or_else(|| problem("missing IFCX header"))?;
    if header.get("ifcxVersion").and_then(Value::as_str) != Some("ifcx_alpha") {
        return Err(problem("unsupported IFCX version"));
    }
    let header: IfcxCadHeader = serde_json::from_value(header.clone())
        .map_err(|e| problem(format!("invalid IFCX header: {e}")))?;
    if header.id.is_empty()
        || header.data_version.is_empty()
        || header.author.is_empty()
        || header.timestamp.is_empty()
    {
        return Err(problem("incomplete IFCX header"));
    }
    let nodes = node_map(&raw)?;
    let drawings: Vec<_> = nodes
        .iter()
        .filter(|(_, n)| attr(n, "ifccad::drawing").is_some())
        .collect();
    if drawings.len() != 1 {
        return Err(problem("expected exactly one CAD drawing"));
    }
    let (drawing_path, drawing_node) = drawings[0];
    let drawing_id = numbered(drawing_path, "</cad/d")?;
    let prefix = drawing_prefix(drawing_id);
    let drawing: DrawingValue = required(drawing_node, "ifccad::drawing")?;
    if drawing.profile_version != "0.1.0" {
        return Err(problem("unsupported CAD profile version"));
    }
    if !unit(&drawing.length_unit) {
        return Err(problem("invalid drawing length unit"));
    }
    let mut layer_paths = BTreeMap::new();
    let mut layers = Vec::new();
    let mut block_paths = BTreeMap::new();
    let mut model_path = None;
    for (path, node) in &nodes {
        if attr(node, "ifccad::layer").is_some() {
            let id = numbered(path, &format!("{prefix}/layer/"))?;
            let value: LayerValue = required(node, "ifccad::layer")?;
            if value.name.is_empty() {
                return Err(problem(format!("{path} empty layer name")));
            }
            appearance(&value.appearance, path)?;
            layer_paths.insert(path.clone(), id);
            layers.push(IfcxCadLayer {
                id,
                name: value.name,
                appearance: value.appearance,
            });
        }
        if attr(node, "ifccad::blockDefinition").is_some() {
            block_paths.insert(path.clone(), numbered(path, &format!("{prefix}/block/"))?);
        }
        if attr(node, "ifccad::layout").is_some() {
            let layout: LayoutValue = required(node, "ifccad::layout")?;
            if layout.kind != "Model" || model_path.replace(path.clone()).is_some() {
                return Err(problem(
                    "expected exactly one Model layout; paper layouts unsupported",
                ));
            }
            numbered(path, &format!("{prefix}/layout/"))?;
        }
    }
    let model_path = model_path.ok_or_else(|| problem("missing Model layout"))?;
    let declared: BTreeSet<_> = drawing_node
        .get("children")
        .and_then(Value::as_object)
        .ok_or_else(|| problem("drawing needs children map"))?
        .values()
        .filter_map(Value::as_str)
        .collect();
    for path in layer_paths
        .keys()
        .chain(block_paths.keys())
        .chain(std::iter::once(&model_path))
    {
        if !declared.contains(path.as_str()) {
            return Err(problem(format!("drawing does not reference {path}")));
        }
    }
    let mut owners = BTreeSet::new();
    let mut parse_owner = |path: &str| -> Result<Vec<IfcxCadEntity>, IfcxCadReport> {
        let node = nodes
            .get(path)
            .ok_or_else(|| problem(format!("missing owner {path}")))?;
        children(node)?
            .iter()
            .map(|target| {
                if !owners.insert(target.clone()) {
                    return Err(problem(format!("entity {target} has more than one owner")));
                }
                let child = nodes
                    .get(target)
                    .ok_or_else(|| problem(format!("{path} missing child {target}")))?;
                entity(child, &prefix, &layer_paths, &block_paths)
            })
            .collect()
    };
    let model = IfcxCadLayout {
        id: numbered(&model_path, &format!("{prefix}/layout/"))?,
        entities: parse_owner(&model_path)?,
    };
    let mut blocks = Vec::new();
    for (path, id) in &block_paths {
        let node = nodes[path];
        let value: DefinitionValue = required(node, "ifccad::blockDefinition")?;
        if value.name.is_empty() || !unit(&value.insertion_unit) {
            return Err(problem(format!("{path} invalid block definition")));
        }
        finite3(value.base_point, path)?;
        blocks.push(IfcxCadBlockDefinition {
            id: *id,
            name: value.name,
            base_point: value.base_point,
            insertion_unit: value.insertion_unit,
            entities: parse_owner(path)?,
        });
    }
    if cycle(&blocks) {
        return Err(problem("block definition cycle"));
    }
    let entity_paths: BTreeSet<_> = nodes
        .iter()
        .filter(|(_, n)| attr(n, "ifccad::entity").is_some())
        .map(|(p, _)| p)
        .collect();
    if entity_paths.len() != owners.len() || entity_paths.iter().any(|p| !owners.contains(*p)) {
        return Err(problem("CAD entity without exactly one owner"));
    }
    Ok(ValidatedIfcxCad {
        raw,
        document: IfcxCadDocument {
            header,
            drawing_id,
            length_unit: drawing.length_unit,
            layers,
            model,
            blocks,
        },
    })
}
