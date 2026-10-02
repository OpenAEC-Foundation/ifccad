use super::*;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DrawingValue {
    profile_version: String,
    length_unit: String,
    next_entity_id: u64,
    next_layer_id: u64,
    next_layout_id: u64,
    next_block_id: u64,
    next_line_pattern_id: u64,
    #[serde(default = "super::patterns::one")]
    line_pattern_scale: f64,
}
#[derive(Deserialize)]
struct LayoutValue {
    kind: String,
    name: Option<String>,
    paper: Option<IfcxCadPaperSize>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LayerValue {
    name: String,
    appearance: super::wire::LayerAppearance,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DefinitionValue {
    name: String,
    base_point: [f64; 3],
    insertion_unit: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EntityValue {
    #[serde(default = "super::patterns::one")]
    line_pattern_scale: f64,
    layer: String,
    appearance: super::wire::EntityAppearance,
}
#[derive(Deserialize)]
struct LineValue {
    start: [f64; 3],
    end: [f64; 3],
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PolylineValue {
    #[serde(default)]
    line_pattern_generation: IfcxCadLinePatternGeneration,
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
    let path = node["path"].as_str().unwrap_or("(unknown path)");
    let value = attr(node, key).ok_or_else(|| problem(format!("{path} missing {key}")))?;
    serde_json::from_value(value.clone()).map_err(|e| problem(format!("{path} invalid {key}: {e}")))
}
fn numbered(path: &str, prefix: &str) -> Result<u64, IfcxCadReport> {
    let suffix = path
        .strip_prefix(prefix)
        .ok_or_else(|| problem(format!("invalid CAD path {path}; expected {prefix}N")))?;
    let number = suffix
        .parse::<u64>()
        .map_err(|_| problem(format!("invalid CAD path {path}")))?;
    if number.to_string() != suffix {
        return Err(problem(format!("noncanonical CAD path {path}")));
    }
    Ok(number)
}
fn drawing_prefix(id: u64) -> String {
    format!("/cad/d{id}")
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
    let path = node["path"].as_str().unwrap_or("(unknown path)");
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
fn entity(
    node: &Value,
    prefix: &str,
    layer_paths: &BTreeMap<String, u64>,
    block_paths: &BTreeMap<String, u64>,
    pattern_paths: &BTreeMap<String, IfcxCadLinePatternId>,
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
    let appearance = value.appearance.typed(pattern_paths)?;
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
            IfcxCadEntityKind::LineSegment {
                start: line.start,
                end: line.end,
            }
        }
        "ifccad::geom::planarPolyline" => {
            let poly: PolylineValue = required(node, present[0])?;
            let frame: IfcxCadPlacement = required(node, "ifccad::geom::placement")?;
            IfcxCadEntityKind::PlanarPolyline {
                line_pattern_generation: poly.line_pattern_generation,
                vertices: poly.vertices,
                closed: poly.closed,
                placement: frame,
            }
        }
        "ifccad::geom::circle" => {
            let circle: CircleValue = required(node, present[0])?;
            let frame: IfcxCadPlacement = required(node, "ifccad::geom::placement")?;
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
            IfcxCadEntityKind::BlockInstance {
                definition_id,
                transform: instance.transform,
            }
        }
    };
    Ok(IfcxCadEntity {
        id,
        layer_id,
        appearance,
        line_pattern_scale: value.line_pattern_scale,
        kind,
    })
}
pub(super) fn project(raw: &Value) -> Result<IfcxCadDocument, IfcxCadReport> {
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
    let nodes = node_map(raw)?;
    let drawings: Vec<_> = nodes
        .iter()
        .filter(|(_, n)| attr(n, "ifccad::drawing").is_some())
        .collect();
    if drawings.len() != 1 {
        return Err(problem("expected exactly one CAD drawing"));
    }
    let (drawing_path, drawing_node) = drawings[0];
    let drawing_id = numbered(drawing_path, "/cad/d")?;
    let prefix = drawing_prefix(drawing_id);
    let drawing: DrawingValue = required(drawing_node, "ifccad::drawing")?;
    if drawing.profile_version != "0.1.0" {
        return Err(problem("unsupported CAD profile version"));
    }
    let mut pattern_paths = BTreeMap::new();
    let mut line_patterns = Vec::new();
    for (path, node) in &nodes {
        if let Some(value) = attr(node, "ifccad::linePattern") {
            #[derive(Deserialize)]
            struct Definition {
                name: String,
                description: Option<String>,
                pattern: Vec<f64>,
            }
            let value: Definition = serde_json::from_value(value.clone())
                .map_err(|e| problem(format!("{path} invalid line pattern: {e}")))?;
            let id = IfcxCadLinePatternId(numbered(path, &format!("{prefix}/linePattern/"))?);
            pattern_paths.insert(path.clone(), id);
            line_patterns.push(IfcxCadLinePattern {
                id,
                name: value.name,
                description: value.description,
                pattern: value.pattern,
            });
        }
    }
    let mut layer_paths = BTreeMap::new();
    let mut layers = Vec::new();
    let mut block_paths = BTreeMap::new();
    let mut model_path = None;
    let mut paper_paths = BTreeMap::new();
    for (path, node) in &nodes {
        if attr(node, "ifccad::layer").is_some() {
            let id = numbered(path, &format!("{prefix}/layer/"))?;
            let value: LayerValue = required(node, "ifccad::layer")?;
            let appearance = value.appearance.typed(&pattern_paths)?;
            layer_paths.insert(path.clone(), id);
            layers.push(IfcxCadLayer {
                id,
                name: value.name,
                appearance,
            });
        }
        if attr(node, "ifccad::blockDefinition").is_some() {
            block_paths.insert(path.clone(), numbered(path, &format!("{prefix}/block/"))?);
        }
        if attr(node, "ifccad::layout").is_some() {
            let layout: LayoutValue = required(node, "ifccad::layout")?;
            let id = numbered(path, &format!("{prefix}/layout/"))?;
            match layout.kind.as_str() {
                "Model" => {
                    if layout.name.is_some() || layout.paper.is_some() {
                        return Err(problem(format!("{path} Model layout has paper metadata")));
                    }
                    if model_path.replace(path.clone()).is_some() {
                        return Err(problem("expected exactly one Model layout"));
                    }
                }
                "Paper" => {
                    let name = layout
                        .name
                        .ok_or_else(|| problem(format!("{path} Paper layout needs a name")))?;
                    let paper = layout.paper.ok_or_else(|| {
                        problem(format!(
                            "{path} Paper layout needs paper dimensions and unit"
                        ))
                    })?;
                    paper_paths.insert(id, (path.clone(), name, paper));
                }
                _ => {
                    return Err(problem(format!(
                        "{path} unsupported layout kind {}",
                        layout.kind
                    )))
                }
            }
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
        .chain(pattern_paths.keys())
        .chain(std::iter::once(&model_path))
        .chain(paper_paths.values().map(|(path, _, _)| path))
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
                entity(child, &prefix, &layer_paths, &block_paths, &pattern_paths)
            })
            .collect()
    };
    let model = IfcxCadLayout {
        id: numbered(&model_path, &format!("{prefix}/layout/"))?,
        entities: parse_owner(&model_path)?,
    };
    let mut paper_layouts = Vec::new();
    for (id, (path, name, paper)) in paper_paths {
        paper_layouts.push(IfcxCadPaperLayout {
            id,
            name,
            paper,
            entities: parse_owner(&path)?,
        });
    }
    let mut blocks = Vec::new();
    for (path, id) in &block_paths {
        let node = nodes[path];
        let value: DefinitionValue = required(node, "ifccad::blockDefinition")?;
        blocks.push(IfcxCadBlockDefinition {
            id: *id,
            name: value.name,
            base_point: value.base_point,
            insertion_unit: value.insertion_unit,
            entities: parse_owner(path)?,
        });
    }
    let entity_paths: BTreeSet<_> = nodes
        .iter()
        .filter(|(_, n)| attr(n, "ifccad::entity").is_some())
        .map(|(p, _)| p)
        .collect();
    if entity_paths.len() != owners.len() || entity_paths.iter().any(|p| !owners.contains(*p)) {
        return Err(problem("CAD entity without exactly one owner"));
    }
    let document = IfcxCadDocument {
        header,
        drawing_id,
        id_counters: IfcxCadIdCounters {
            next_entity_id: drawing.next_entity_id,
            next_layer_id: drawing.next_layer_id,
            next_layout_id: drawing.next_layout_id,
            next_block_id: drawing.next_block_id,
            next_line_pattern_id: drawing.next_line_pattern_id,
        },
        length_unit: drawing.length_unit,
        line_patterns,
        line_pattern_scale: drawing.line_pattern_scale,
        layers,
        model,
        paper_layouts,
        blocks,
    };
    validate_ifcx_cad_document(&document)?;
    Ok(document)
}
