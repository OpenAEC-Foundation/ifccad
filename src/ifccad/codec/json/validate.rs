fn initial_workspace_id() -> u64 {
    1
}
use super::*;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DrawingValue {
    profile_version: String,
    length_unit: String,
    next_entity_id: u64,
    next_layer_id: u64,
    next_layout_id: u64,
    next_block_id: u64,
    next_line_pattern_id: u64,
    next_preservation_record_id: Option<u64>,
    #[serde(default = "initial_workspace_id")]
    next_ucs_id: u64,
    #[serde(default = "initial_workspace_id")]
    next_model_window_id: u64,
    #[serde(default = "crate::ifccad::logical::patterns::one")]
    line_pattern_scale: f64,
    plot_style_mode: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LayoutValue {
    kind: String,
    tab_index: u32,
    name: Option<String>,
    media: Option<Value>,
    limits: Option<Value>,
    limits_checking: Option<bool>,
    paper_space_linetype_scaling: Option<bool>,
    plot_settings: Option<Value>,
    bounds: Option<IfccadBounds3d>,
}
impl LayoutValue {
    fn output(&self) -> Option<IfccadLayoutSettings> {
        let mut v = serde_json::json!({});
        for (key, value) in [
            ("media", &self.media),
            ("limits", &self.limits),
            ("plotSettings", &self.plot_settings),
        ] {
            if let Some(value) = value {
                v[key] = value.clone();
            }
        }
        if let Some(b) = self.limits_checking {
            v["limitsChecking"] = serde_json::json!(b);
        }
        if let Some(b) = self.paper_space_linetype_scaling {
            v["paperSpaceLinetypeScaling"] = serde_json::json!(b);
        }
        super::layout::decode_output(&v)
    }
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
    bounds: Option<IfccadBounds3d>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EntityValue {
    #[serde(default = "crate::ifccad::logical::patterns::one")]
    line_pattern_scale: f64,
    layer: String,
    appearance: super::wire::EntityAppearance,
}
#[derive(Deserialize)]
struct InstanceValue {
    definition: String,
    transform: IfccadBlockTransform,
}

fn problem(message: impl Into<String>) -> IfccadReport {
    IfccadReport::one(message)
}
fn preservation_node_role(node: &Value, key: &str) -> Result<(), IfccadReport> {
    let attributes = node["attributes"]
        .as_object()
        .ok_or_else(|| problem("preservation node attributes required"))?;
    if attributes
        .keys()
        .any(|name| name.starts_with("ifccad::") && name != key)
    {
        return Err(problem(
            "preservation node has an incompatible CAD attribute role",
        ));
    }
    Ok(())
}
fn attr<'a>(node: &'a Value, key: &str) -> Option<&'a Value> {
    node.get("attributes")?.get(key)
}
fn required<T: for<'de> Deserialize<'de>>(node: &Value, key: &str) -> Result<T, IfccadReport> {
    let path = node["path"].as_str().unwrap_or("(unknown path)");
    let value = attr(node, key).ok_or_else(|| problem(format!("{path} missing {key}")))?;
    serde_json::from_value(value.clone()).map_err(|e| problem(format!("{path} invalid {key}: {e}")))
}
pub(super) fn numbered(path: &str, prefix: &str) -> Result<u64, IfccadReport> {
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
fn node_map(raw: &Value) -> Result<BTreeMap<String, &Value>, IfccadReport> {
    let mut nodes = BTreeMap::new();
    for node in raw["data"]
        .as_array()
        .ok_or_else(|| problem("IFCX data must be an array"))?
    {
        for key in ["ifccad::layout", "ifccad::blockDefinition"] {
            if attr(node, key)
                .and_then(|v| v.get("bounds"))
                .is_some_and(Value::is_null)
            {
                return Err(problem(
                    "explicit null bounds are invalid; omit absent bounds",
                ));
            }
        }

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
fn children(node: &Value) -> Result<Vec<String>, IfccadReport> {
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
    pattern_paths: &BTreeMap<String, IfccadLinePatternId>,
) -> Result<IfccadEntity, IfccadReport> {
    let path = node["path"].as_str().unwrap();
    let id = numbered(path, &format!("{prefix}/e"))?;
    if let Some(value) = attr(node, "ifccad::opaqueEntity") {
        let attributes = node["attributes"]
            .as_object()
            .ok_or_else(|| problem("opaque attributes required"))?;
        if attributes
            .keys()
            .any(|k| k.starts_with("ifccad::") && k != "ifccad::opaqueEntity")
        {
            return Err(problem("opaque and native attributes cannot coexist"));
        }
        return super::preservation::decode_opaque(value, id, prefix, pattern_paths);
    }
    appearance_modes(&node["attributes"]["ifccad::entity"]["appearance"], path)?;
    let value: EntityValue = required(node, "ifccad::entity")?;
    let layer_id = *layer_paths
        .get(&value.layer)
        .ok_or_else(|| problem(format!("{path} unresolved layer {}", value.layer)))?;
    let appearance = value.appearance.typed(pattern_paths)?;
    let attrs = node
        .get("attributes")
        .and_then(Value::as_object)
        .ok_or_else(|| problem(format!("{path} missing attributes")))?;
    let mut payloads = super::geometry::PAYLOADS.to_vec();
    payloads.extend(["ifccad::viewport", "ifccad::blockInstance"]);
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
        "ifccad::viewport" => {
            if attrs.contains_key("ifccad::geom::placement") {
                return Err(problem(format!("{path} viewport has a geometry placement")));
            }
            IfccadEntityKind::Viewport(super::viewports::decode_viewport(
                &attrs["ifccad::viewport"],
                prefix,
            )?)
        }
        "ifccad::blockInstance" => {
            let instance: InstanceValue = required(node, present[0])?;
            let definition_id = *block_paths.get(&instance.definition).ok_or_else(|| {
                problem(format!(
                    "{path} unresolved block definition {}",
                    instance.definition
                ))
            })?;
            IfccadEntityKind::BlockInstance {
                definition_id,
                transform: instance.transform,
            }
        }
        key => super::geometry::decode_kind(attrs, key, path)?,
    };
    Ok(IfccadEntity::Native(IfccadNativeEntity {
        id,
        layer_id,
        appearance,
        line_pattern_scale: value.line_pattern_scale,
        kind,
    }))
}
pub(super) fn appearance_modes(value: &Value, path: &str) -> Result<(), IfccadReport> {
    let modes = value
        .as_object()
        .ok_or_else(|| problem(format!("{path} missing entity appearance")))?;
    if modes.len() != 4 {
        return Err(problem(format!(
            "{path} appearance requires exactly four native properties"
        )));
    }
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
    Ok(())
}
pub(crate) fn project(raw: &Value) -> Result<IfccadDocument, IfccadReport> {
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
        "../../../../schemas/ifccad/ifccad-profile-0.1.0.ifcx"
    ))
    .map_err(|e| problem(format!("invalid built-in CAD schema module: {e}")))?;
    let known = profile_module["schemas"]
        .as_object()
        .expect("built-in schema module has schemas");
    let mut previous_drawing_schema = known["ifccad::drawing"].clone();
    previous_drawing_schema["value"]["objectRestrictions"]["values"]
        .as_object_mut()
        .unwrap()
        .remove("nextPreservationRecordId");
    let native_only = raw["data"].as_array().is_some_and(|nodes| {
        nodes.iter().all(|n| {
            [
                "ifccad::opaqueEntity",
                "ifccad::preservation",
                "ifccad::preservationRecord",
            ]
            .iter()
            .all(|key| attr(n, key).is_none())
        })
    });
    for node in raw["data"]
        .as_array()
        .ok_or_else(|| problem("IFCX data must be an array"))?
    {
        if let Some(attributes) = node.get("attributes").and_then(Value::as_object) {
            for key in attributes.keys().filter(|key| key.starts_with("ifccad::")) {
                if let Some(expected) = known.get(key) {
                    if schemas.get(key).is_some_and(|actual| {
                        actual != expected
                            && !(key == "ifccad::drawing"
                                && native_only
                                && attributes[key].get("nextPreservationRecordId").is_none()
                                && actual == &previous_drawing_schema)
                    }) || (schemas.get(key).is_none() && !imported_profile)
                    {
                        return Err(problem(format!("missing or changed CAD schema {key}")));
                    }
                } else {
                    return Err(problem(format!("unsupported CAD profile attribute {key}")));
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
    let header: IfccadHeader = serde_json::from_value(header.clone())
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
            let id = IfccadLinePatternId(numbered(path, &format!("{prefix}/linePattern/"))?);
            pattern_paths.insert(path.clone(), id);
            line_patterns.push(IfccadLinePattern {
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
            layers.push(IfccadLayer {
                id,
                name: value.name,
                appearance,
            });
        }
        if attr(node, "ifccad::blockDefinition").is_some() {
            block_paths.insert(path.clone(), numbered(path, &format!("{prefix}/block/"))?);
        }
        if attr(node, "ifccad::layout").is_some() {
            let raw = attr(node, "ifccad::layout").unwrap();
            if [
                "media",
                "limits",
                "plotSettings",
                "limitsChecking",
                "paperSpaceLinetypeScaling",
            ]
            .into_iter()
            .any(|key| raw.get(key).is_some_and(Value::is_null))
            {
                return Err(problem(format!(
                    "{path} layout output fields cannot be null"
                )));
            }
            let layout: LayoutValue = required(node, "ifccad::layout")?;
            let id = numbered(path, &format!("{prefix}/layout/"))?;
            match layout.kind.as_str() {
                "Model" => {
                    if ["name"].iter().any(|key| {
                        attr(node, "ifccad::layout").is_some_and(|v| v.get(key).is_some())
                    }) || layout.tab_index != 0
                    {
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
                    paper_paths.insert(id, (path.clone(), name, layout.tab_index));
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
        .chain(paper_paths.values().map(|(path, ..)| path))
    {
        if !declared.contains(path.as_str()) {
            return Err(problem(format!("drawing does not reference {path}")));
        }
    }
    let mut owners = BTreeSet::new();
    let mut parse_owner = |path: &str| -> Result<Vec<IfccadEntity>, IfccadReport> {
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
    let model = IfccadLayout {
        settings: required::<LayoutValue>(nodes[&model_path], "ifccad::layout")?
            .output()
            .ok_or_else(|| problem("invalid Model layout output"))?,
        bounds: required::<LayoutValue>(nodes[&model_path], "ifccad::layout")?.bounds,
        id: numbered(&model_path, &format!("{prefix}/layout/"))?,
        tab_index: 0,
        entities: parse_owner(&model_path)?,
    };
    let mut paper_layouts = Vec::new();
    for (id, (path, name, tab_index)) in paper_paths {
        paper_layouts.push(IfccadPaperLayout {
            canvas: None,

            bounds: required::<LayoutValue>(nodes[&path], "ifccad::layout")?.bounds,
            id,
            name,
            tab_index,
            settings: required::<LayoutValue>(nodes[&path], "ifccad::layout")?
                .output()
                .ok_or_else(|| problem("invalid Paper layout output"))?,
            entities: parse_owner(&path)?,
        });
    }
    paper_layouts.sort_by_key(|layout| layout.tab_index);
    let mut blocks = Vec::new();
    for (path, id) in &block_paths {
        let node = nodes[path];
        let value: DefinitionValue = required(node, "ifccad::blockDefinition")?;
        blocks.push(IfccadBlockDefinition {
            bounds: value.bounds,
            id: *id,
            name: value.name,
            base_point: value.base_point,
            insertion_unit: value.insertion_unit,
            entities: parse_owner(path)?,
        });
    }
    let entity_paths: BTreeSet<_> = nodes
        .iter()
        .filter(|(_, n)| {
            attr(n, "ifccad::entity").is_some() || attr(n, "ifccad::opaqueEntity").is_some()
        })
        .map(|(p, _)| p)
        .collect();
    if entity_paths.len() != owners.len() || entity_paths.iter().any(|p| !owners.contains(*p)) {
        return Err(problem("CAD entity without exactly one owner"));
    }
    if attr(drawing_node, "ifccad::drawing")
        .and_then(|v| v.get("plotStyleMode"))
        .is_some_and(Value::is_null)
    {
        return Err(problem("plotStyleMode cannot be null"));
    }
    let preservation_path = format!("{prefix}/preservation");
    let preservation = if let Some(node) = nodes.get(&preservation_path) {
        if !declared.contains(preservation_path.as_str()) {
            return Err(problem("drawing must reference preservation collection"));
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Collection {
            version: u32,
            sources: Vec<IfccadPreservationSource>,
        }
        preservation_node_role(node, "ifccad::preservation")?;
        let collection: Collection = required(node, "ifccad::preservation")?;
        if attr(node, "ifccad::preservation")
            .and_then(|v| v.get("sources"))
            .and_then(Value::as_array)
            .is_some_and(|sources| {
                sources
                    .iter()
                    .any(|s| s.get("sourceVersion").is_some_and(Value::is_null))
            })
        {
            return Err(problem("omit absent source version"));
        }
        if collection
            .sources
            .iter()
            .any(|s| s.source_version.as_ref().is_some_and(String::is_empty))
        {
            return Err(problem("invalid source version"));
        }
        let record_children = node["children"]
            .as_object()
            .ok_or_else(|| problem("preservation children map required"))?;
        let mut records = Vec::new();
        let mut declared_records = BTreeSet::new();
        for (key, path) in record_children {
            let path = path
                .as_str()
                .ok_or_else(|| problem("record child path required"))?;
            let id = numbered(path, &format!("{prefix}/preservation/r"))?;
            if key != &format!("r{id}") || !declared_records.insert(path) {
                return Err(problem("noncanonical or duplicate record child"));
            }
            let node = nodes
                .get(path)
                .ok_or_else(|| problem("missing preservation record node"))?;
            preservation_node_role(node, "ifccad::preservationRecord")?;
            let value = attr(node, "ifccad::preservationRecord")
                .ok_or_else(|| problem("record attribute required"))?;
            records.push(super::preservation::decode_record(value, id, &prefix)?);
        }
        if nodes.iter().any(|(path, node)| {
            attr(node, "ifccad::preservationRecord").is_some()
                && !declared_records.contains(path.as_str())
        }) {
            return Err(problem("unowned preservation record"));
        }
        Some(IfccadPreservation {
            version: collection.version,
            sources: collection.sources,
            records,
        })
    } else {
        if nodes.values().any(|node| {
            attr(node, "ifccad::preservationRecord").is_some()
                || attr(node, "ifccad::preservation").is_some()
        }) {
            return Err(problem("misplaced preservation node"));
        }
        None
    };
    if attr(drawing_node, "ifccad::drawing")
        .and_then(|v| v.get("nextPreservationRecordId"))
        .is_some_and(Value::is_null)
    {
        return Err(problem("omit absent preservation watermark"));
    }
    let next_preservation_record_id = match drawing.next_preservation_record_id {
        Some(id) => id,
        None if preservation.is_none() => 1,
        _ => return Err(problem("preservation requires allocation watermark")),
    };
    let mut document = IfccadDocument {
        ucs_definitions: vec![],
        model_windows: vec![],
        workspace_state: None,
        model_view_state: None,
        preservation,
        header,
        drawing_id,
        id_counters: IfccadIdCounters {
            next_preservation_record_id,

            next_ucs_id: drawing.next_ucs_id,
            next_model_window_id: drawing.next_model_window_id,

            next_entity_id: drawing.next_entity_id,
            next_layer_id: drawing.next_layer_id,
            next_layout_id: drawing.next_layout_id,
            next_block_id: drawing.next_block_id,
            next_line_pattern_id: drawing.next_line_pattern_id,
        },
        length_unit: drawing.length_unit,
        plot_style_mode: match drawing.plot_style_mode.as_deref() {
            None | Some("colorDependent") => IfccadPlotStyleMode::ColorDependent,
            Some("named") => IfccadPlotStyleMode::Named,
            _ => return Err(problem("invalid plot style mode")),
        },
        line_patterns,
        line_pattern_scale: drawing.line_pattern_scale,
        layers,
        model,
        paper_layouts,
        blocks,
    };
    super::workspace::decode_document(raw, &mut document)?;
    validate_ifccad_document(&document)?;
    Ok(document)
}
