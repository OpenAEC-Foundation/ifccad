//! Canonical owned graph projection, sharing the closed family value adapters.
use super::encode::NodeOut;
use super::leader_values::LeaderJsonValue;
use super::*;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

const OWNED: &[&str] = &[
    "ifccad::leaderGraphics",
    "ifccad::leaderGraphicsCommand",
    "ifccad::multiLeaderPresentation",
    "ifccad::multiLeaderBranches",
    "ifccad::multiLeaderBranch",
    "ifccad::multiLeaderLine",
    "ifccad::multiLeaderContent",
    "ifccad::multiLeaderText",
    "ifccad::multiLeaderBlock",
    "ifccad::multiLeaderTolerance",
    "ifccad::multiLeaderBlockLabels",
    "ifccad::multiLeaderBlockLabel",
];
fn fail(path: &str, detail: &str) -> IfccadReport {
    crate::ifccad::diagnostics::failure("IFCCAD-LEADER-GRAPH-001", path, detail)
}
fn remove(value: &mut Value, name: &str) -> Option<Value> {
    value.as_object_mut().expect("typed record").remove(name)
}
fn node(path: &str, role: &str, value: Value, children: Option<Map<String, Value>>) -> NodeOut {
    NodeOut {
        path: path.into(),
        children,
        attributes: Map::from_iter([(role.into(), value)]),
    }
}
fn link(children: &mut Map<String, Value>, name: impl Into<String>, path: &str) {
    children.insert(name.into(), json!(path));
}
fn push_graphics(mut value: Value, path: &str, out: &mut Vec<NodeOut>) {
    let commands = remove(&mut value, "commands").expect("typed commands");
    let mut children = Map::new();
    for (index, command) in commands.as_array().expect("typed array").iter().enumerate() {
        let target = format!("{path}/op{index}");
        link(&mut children, index.to_string(), &target);
        out.push(node(
            &target,
            "ifccad::leaderGraphicsCommand",
            command.clone(),
            None,
        ));
    }
    out.push(node(path, "ifccad::leaderGraphics", value, Some(children)));
}
fn push_presentation(
    mut value: Value,
    path: &str,
    out: &mut Vec<NodeOut>,
) -> (Value, Map<String, Value>) {
    let groups = remove(&mut value, "groups").expect("typed groups");
    let content = remove(&mut value, "content").expect("typed content");
    let mut children = Map::new();
    let branches = format!("{path}/leaders");
    link(&mut children, "leaders", &branches);
    let mut roots = Map::new();
    for (index, group) in groups.as_array().expect("typed groups").iter().enumerate() {
        let mut group = group.clone();
        let group_path = format!("{branches}/r{}", group["key"].as_u64().expect("typed key"));
        link(&mut roots, index.to_string(), &group_path);
        let lines = remove(&mut group, "lines").expect("typed lines");
        let mut line_children = Map::new();
        for (index, line) in lines.as_array().expect("typed lines").iter().enumerate() {
            let target = format!("{group_path}/l{}", line["key"].as_u64().expect("typed key"));
            link(&mut line_children, index.to_string(), &target);
            out.push(node(&target, "ifccad::multiLeaderLine", line.clone(), None));
        }
        out.push(node(
            &group_path,
            "ifccad::multiLeaderBranch",
            group,
            Some(line_children),
        ));
    }
    out.push(node(
        &branches,
        "ifccad::multiLeaderBranches",
        json!({}),
        Some(roots),
    ));
    if !content.as_object().expect("typed content").is_empty() {
        let content_path = format!("{path}/content");
        link(&mut children, "content", &content_path);
        let mut parts = Map::new();
        for (name, role) in [
            ("mText", "ifccad::multiLeaderText"),
            ("block", "ifccad::multiLeaderBlock"),
            ("tolerance", "ifccad::multiLeaderTolerance"),
        ] {
            if let Some(value) = content.get(name) {
                let target = format!("{content_path}/{name}");
                link(&mut parts, name, &target);
                out.push(node(&target, role, value.clone(), None));
            }
        }
        out.push(node(
            &content_path,
            "ifccad::multiLeaderContent",
            json!({}),
            Some(parts),
        ));
    }
    (value, children)
}
pub(super) fn write_multi_leader(
    leader: &IfccadMultiLeader,
    owner: &str,
    prefix: &str,
) -> Vec<NodeOut> {
    let mut value = leader.write_value(prefix);
    let mut out = vec![];
    let (presentation, mut children) = push_presentation(
        remove(&mut value, "presentation").expect("typed presentation"),
        owner,
        &mut out,
    );
    value["presentation"] = presentation;
    let labels = remove(&mut value, "blockLabels").expect("typed labels");
    if !labels.as_array().expect("typed labels").is_empty() {
        let path = format!("{owner}/labels");
        link(&mut children, "blockLabels", &path);
        let mut items = Map::new();
        for (index, label) in labels.as_array().unwrap().iter().enumerate() {
            let target = format!("{path}/a{}", label["key"].as_u64().expect("typed key"));
            link(&mut items, index.to_string(), &target);
            out.push(node(
                &target,
                "ifccad::multiLeaderBlockLabel",
                label.clone(),
                None,
            ));
        }
        out.push(node(
            &path,
            "ifccad::multiLeaderBlockLabels",
            json!({}),
            Some(items),
        ));
    }
    let mut annotation = remove(&mut value, "annotation");
    if let Some(annotation) = &mut annotation {
        for context in annotation["contexts"]
            .as_array_mut()
            .expect("typed contexts")
        {
            if let Some(presentation) = remove(context, "presentation") {
                let scale = context["key"]["scaleId"]
                    .as_str()
                    .expect("typed scale")
                    .rsplit('/')
                    .next()
                    .unwrap();
                let name = format!("context{scale}");
                let path = format!("{owner}/{name}");
                link(&mut children, name, &path);
                let (body, parts) = push_presentation(presentation, &path, &mut out);
                out.push(node(
                    &path,
                    "ifccad::multiLeaderPresentation",
                    body,
                    Some(parts),
                ));
                context["presentation"] = json!({"graph":path});
            }
        }
    }
    if let Some(graphics) = remove(&mut value, "graphics") {
        let path = format!("{owner}/graphics");
        link(&mut children, "graphics", &path);
        push_graphics(graphics, &path, &mut out);
    }
    let mut root = node(owner, "ifccad::multiLeader", value, Some(children));
    if let Some(annotation) = annotation {
        root.attributes
            .insert("ifccad::annotation".into(), annotation);
    }
    out.insert(0, root);
    out
}
fn write_classic(mut value: Value, owner: &str, role: &str) -> Vec<NodeOut> {
    let mut out = vec![];
    let mut children = Map::new();
    if let Some(graphics) = remove(&mut value, "graphics") {
        let path = format!("{owner}/graphics");
        link(&mut children, "graphics", &path);
        push_graphics(graphics, &path, &mut out);
    }
    let annotation = remove(&mut value, "annotation");
    let placement = remove(&mut value, "placement").expect("typed placement");
    let mut root = node(
        owner,
        role,
        value,
        if children.is_empty() {
            None
        } else {
            Some(children)
        },
    );
    root.attributes
        .insert("ifccad::geom::placement".into(), placement);
    if let Some(annotation) = annotation {
        root.attributes
            .insert("ifccad::annotation".into(), annotation);
    }
    out.insert(0, root);
    out
}
pub(super) fn write_leader(value: &IfccadLeader, owner: &str, prefix: &str) -> Vec<NodeOut> {
    write_classic(value.write_value(prefix), owner, "ifccad::leader")
}
pub(super) fn write_tolerance(value: &IfccadTolerance, owner: &str, prefix: &str) -> Vec<NodeOut> {
    write_classic(value.write_value(prefix), owner, "ifccad::tolerance")
}

pub(super) fn style_value(style: &IfccadMultiLeaderStyle, prefix: &str) -> Value {
    let mut value = style.write_value(prefix);
    remove(&mut value, "id");
    value
}
pub(super) fn read_style(
    value: &Value,
    id: IfccadMultiLeaderStyleId,
    prefix: &str,
    path: &str,
) -> Result<IfccadMultiLeaderStyle, IfccadReport> {
    super::supplemental::validate_value("ifccad::multiLeaderStyle", value, path)?;
    let mut value = value.clone();
    let object = value
        .as_object_mut()
        .ok_or_else(|| fail(path, "style object required"))?;
    if object.insert("id".into(), id.write_value(prefix)).is_some() {
        return Err(fail(path, "style identity belongs to its node path"));
    }
    IfccadMultiLeaderStyle::read_value(&value, prefix, path)
}

pub(super) struct Reader<'a> {
    nodes: &'a BTreeMap<String, &'a Value>,
    prefix: &'a str,
    seen: BTreeSet<String>,
}
impl<'a> Reader<'a> {
    pub(super) fn new(nodes: &'a BTreeMap<String, &'a Value>, prefix: &'a str) -> Self {
        Self {
            nodes,
            prefix,
            seen: BTreeSet::new(),
        }
    }
    fn role(
        &mut self,
        path: &str,
        role: &str,
        extra: &[&str],
        container: bool,
    ) -> Result<(Value, Map<String, Value>), IfccadReport> {
        let node = self
            .nodes
            .get(path)
            .ok_or_else(|| fail(path, "missing owned node"))?;
        if !self.seen.insert(path.into()) {
            return Err(fail(path, "node has more than one role/owner"));
        }
        let attrs = node["attributes"]
            .as_object()
            .ok_or_else(|| fail(path, "attributes required"))?;
        if attrs
            .keys()
            .any(|name| name != role && !extra.contains(&name.as_str()))
        {
            return Err(fail(path, "incompatible owned role"));
        }
        let value = attrs
            .get(role)
            .ok_or_else(|| fail(path, "wrong owned role"))?
            .clone();
        super::supplemental::validate_value(role, &value, path)?;
        let children = match node.get("children") {
            Some(value) => value
                .as_object()
                .ok_or_else(|| fail(path, "children object required"))?
                .clone(),
            None if !container => Map::new(),
            _ => return Err(fail(path, "children object required")),
        };
        Ok((value, children))
    }
    fn child(
        &self,
        children: &mut Map<String, Value>,
        name: &str,
        path: &str,
        required: bool,
    ) -> Result<bool, IfccadReport> {
        match children.remove(name) {
            Some(value) if value.as_str() == Some(path) => Ok(true),
            Some(_) => Err(fail(path, "noncanonical owned link")),
            None if !required => Ok(false),
            _ => Err(fail(path, "missing owned link")),
        }
    }
    fn no_children(&self, children: &Map<String, Value>, path: &str) -> Result<(), IfccadReport> {
        if children.is_empty() {
            Ok(())
        } else {
            Err(fail(path, "unexpected owned children"))
        }
    }
    fn insert(
        &self,
        value: &mut Value,
        name: &str,
        field: Value,
        path: &str,
    ) -> Result<(), IfccadReport> {
        let object = value
            .as_object_mut()
            .ok_or_else(|| fail(path, "record object required"))?;
        if object.insert(name.into(), field).is_some() {
            return Err(fail(path, "owned field cannot also occur inline"));
        }
        Ok(())
    }
    fn empty(&self, value: &Value, path: &str) -> Result<(), IfccadReport> {
        if value.as_object().is_some_and(Map::is_empty) {
            Ok(())
        } else {
            Err(fail(path, "container payload must be empty"))
        }
    }
    fn indexed(
        &self,
        children: &Map<String, Value>,
        path: &str,
        prefix: &str,
    ) -> Result<Vec<String>, IfccadReport> {
        (0..children.len())
            .map(|index| {
                let value = children
                    .get(&index.to_string())
                    .and_then(Value::as_str)
                    .ok_or_else(|| fail(path, "contiguous numeric children required"))?;
                let suffix = value
                    .strip_prefix(&format!("{path}/{prefix}"))
                    .ok_or_else(|| fail(path, "noncanonical part path"))?;
                let key = suffix
                    .parse::<u64>()
                    .map_err(|_| fail(path, "invalid local part key"))?;
                if suffix != key.to_string() || key == 0 {
                    return Err(fail(path, "noncanonical local part key"));
                }
                Ok(value.to_owned())
            })
            .collect()
    }
    fn key(&self, body: &Value, path: &str, prefix: &str) -> Result<(), IfccadReport> {
        let key = body["key"]
            .as_u64()
            .ok_or_else(|| fail(path, "local key required"))?;
        if !path.ends_with(&format!("/{prefix}{key}")) {
            return Err(fail(path, "part path/key mismatch"));
        }
        Ok(())
    }
    fn graphics(&mut self, path: &str) -> Result<Value, IfccadReport> {
        let (mut body, children) = self.role(path, "ifccad::leaderGraphics", &[], true)?;
        let mut commands = vec![];
        for index in 0..children.len() {
            let target = format!("{path}/op{index}");
            if children.get(&index.to_string()).and_then(Value::as_str) != Some(&target) {
                return Err(fail(path, "contiguous canonical command links required"));
            }
            let (value, children) =
                self.role(&target, "ifccad::leaderGraphicsCommand", &[], false)?;
            self.no_children(&children, &target)?;
            commands.push(value);
        }
        self.insert(&mut body, "commands", Value::Array(commands), path)?;
        Ok(body)
    }
    fn presentation(
        &mut self,
        mut body: Value,
        mut children: Map<String, Value>,
        path: &str,
    ) -> Result<Value, IfccadReport> {
        let branches = format!("{path}/leaders");
        self.child(&mut children, "leaders", &branches, true)?;
        let (empty, groups) = self.role(&branches, "ifccad::multiLeaderBranches", &[], true)?;
        self.empty(&empty, &branches)?;
        let mut output = vec![];
        for target in self.indexed(&groups, &branches, "r")? {
            let (mut group, lines) = self.role(&target, "ifccad::multiLeaderBranch", &[], true)?;
            self.key(&group, &target, "r")?;
            let mut values = vec![];
            for line in self.indexed(&lines, &target, "l")? {
                let (value, children) = self.role(&line, "ifccad::multiLeaderLine", &[], false)?;
                self.key(&value, &line, "l")?;
                self.no_children(&children, &line)?;
                values.push(value);
            }
            self.insert(&mut group, "lines", Value::Array(values), &target)?;
            output.push(group);
        }
        self.insert(&mut body, "groups", Value::Array(output), path)?;
        let content_path = format!("{path}/content");
        let mut content = Map::new();
        if self.child(&mut children, "content", &content_path, false)? {
            let (empty, mut parts) =
                self.role(&content_path, "ifccad::multiLeaderContent", &[], true)?;
            self.empty(&empty, &content_path)?;
            for (name, role) in [
                ("mText", "ifccad::multiLeaderText"),
                ("block", "ifccad::multiLeaderBlock"),
                ("tolerance", "ifccad::multiLeaderTolerance"),
            ] {
                let target = format!("{content_path}/{name}");
                if self.child(&mut parts, name, &target, false)? {
                    let (value, children) = self.role(&target, role, &[], false)?;
                    self.no_children(&children, &target)?;
                    content.insert(name.into(), value);
                }
            }
            self.no_children(&parts, &content_path)?;
            if content.is_empty() {
                return Err(fail(
                    &content_path,
                    "empty content container must be omitted",
                ));
            }
        }
        self.insert(&mut body, "content", Value::Object(content), path)?;
        self.no_children(&children, path)?;
        Ok(body)
    }
    pub(super) fn read_multi_leader(
        &mut self,
        owner: &str,
    ) -> Result<IfccadMultiLeader, IfccadReport> {
        let (mut body, mut children) = self.role(
            owner,
            "ifccad::multiLeader",
            &["ifccad::entity", "ifccad::annotation"],
            true,
        )?;
        let mut annotation = self.nodes[owner]["attributes"]
            .get("ifccad::annotation")
            .cloned();
        if let Some(annotation) = &mut annotation {
            for context in annotation
                .get_mut("contexts")
                .and_then(Value::as_array_mut)
                .ok_or_else(|| fail(owner, "annotation contexts required"))?
            {
                if let Some(reference) = context.get("presentation") {
                    let scale = super::dimension_values::reference(
                        self.prefix,
                        "annotationScale/",
                        context["key"]["scaleId"]
                            .as_str()
                            .ok_or_else(|| fail(owner, "scale reference required"))?,
                        owner,
                    )?;
                    let path = format!("{owner}/context{scale}");
                    if reference.as_object().is_none_or(|object| object.len() != 1)
                        || reference["graph"].as_str() != Some(&path)
                    {
                        return Err(fail(owner, "canonical context graph reference required"));
                    }
                    self.child(&mut children, &format!("context{scale}"), &path, true)?;
                    let (value, parts) =
                        self.role(&path, "ifccad::multiLeaderPresentation", &[], true)?;
                    context["presentation"] = self.presentation(value, parts, &path)?;
                }
            }
        }
        let path = format!("{owner}/labels");
        let mut labels = vec![];
        if self.child(&mut children, "blockLabels", &path, false)? {
            let (empty, items) = self.role(&path, "ifccad::multiLeaderBlockLabels", &[], true)?;
            self.empty(&empty, &path)?;
            for target in self.indexed(&items, &path, "a")? {
                let (value, children) =
                    self.role(&target, "ifccad::multiLeaderBlockLabel", &[], false)?;
                self.key(&value, &target, "a")?;
                self.no_children(&children, &target)?;
                labels.push(value);
            }
            if labels.is_empty() {
                return Err(fail(&path, "empty label container must be omitted"));
            }
        }
        self.insert(&mut body, "blockLabels", Value::Array(labels), owner)?;
        if let Some(annotation) = annotation {
            self.insert(&mut body, "annotation", annotation, owner)?;
        }
        let path = format!("{owner}/graphics");
        if self.child(&mut children, "graphics", &path, false)? {
            let value = self.graphics(&path)?;
            self.insert(&mut body, "graphics", value, owner)?;
        }
        let presentation = remove(&mut body, "presentation")
            .ok_or_else(|| fail(owner, "presentation metadata required"))?;
        body["presentation"] = self.presentation(presentation, children, owner)?;
        IfccadMultiLeader::read_value(&body, self.prefix, owner)
    }
    fn read_classic(&mut self, owner: &str, role: &str) -> Result<Value, IfccadReport> {
        let (mut body, mut children) = self.role(
            owner,
            role,
            &[
                "ifccad::entity",
                "ifccad::annotation",
                "ifccad::geom::placement",
            ],
            false,
        )?;
        let attrs = &self.nodes[owner]["attributes"];
        self.insert(
            &mut body,
            "placement",
            attrs
                .get("ifccad::geom::placement")
                .ok_or_else(|| fail(owner, "placement required"))?
                .clone(),
            owner,
        )?;
        if let Some(annotation) = attrs.get("ifccad::annotation") {
            self.insert(&mut body, "annotation", annotation.clone(), owner)?;
        }
        let path = format!("{owner}/graphics");
        if self.child(&mut children, "graphics", &path, false)? {
            let value = self.graphics(&path)?;
            self.insert(&mut body, "graphics", value, owner)?;
        }
        self.no_children(&children, owner)?;
        Ok(body)
    }
    pub(super) fn read_leader(&mut self, owner: &str) -> Result<IfccadLeader, IfccadReport> {
        let value = self.read_classic(owner, "ifccad::leader")?;
        IfccadLeader::read_value(&value, self.prefix, owner)
    }
    pub(super) fn read_tolerance(&mut self, owner: &str) -> Result<IfccadTolerance, IfccadReport> {
        let value = self.read_classic(owner, "ifccad::tolerance")?;
        IfccadTolerance::read_value(&value, self.prefix, owner)
    }
    pub(super) fn finish(&self) -> Result<(), IfccadReport> {
        for (path, node) in self.nodes {
            if OWNED
                .iter()
                .any(|role| node["attributes"].get(*role).is_some())
            {
                if !self.seen.contains(path) {
                    return Err(fail(path, "unowned leader part"));
                }
                let incoming = self
                    .nodes
                    .values()
                    .filter_map(|node| node.get("children").and_then(Value::as_object))
                    .flat_map(|children| children.values())
                    .filter(|value| value.as_str() == Some(path))
                    .count();
                if incoming != 1 {
                    return Err(fail(
                        path,
                        "owned leader part requires exactly one incoming child link",
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::collections::BTreeMap;
    fn values(leader: &IfccadMultiLeader) -> Vec<Value> {
        write_multi_leader(leader, "/cad/d1/e80", "/cad/d1")
            .into_iter()
            .map(|node| serde_json::to_value(node).unwrap())
            .collect()
    }
    #[test]
    fn owned_graph_roundtrip_keeps_base_contexts_labels_and_commands() {
        let mut leader = IfccadMultiLeader::new();
        leader.block_labels.push(IfccadMultiLeaderBlockLabel {
            key: 3,
            definition_id: None,
            source_index: Some(20),
            width: 0.,
            text: "A-12".into(),
        });
        leader.next_label_id = 4;
        leader.presentation.content.tolerance = Some(Box::new(IfccadMultiLeaderToleranceContent {
            placement: leader.presentation.plane.clone(),
            dimension_style: IfccadDimensionStyleId(7),
            overrides: Default::default(),
            notation: IfccadToleranceNotation::CadFeatureControlTextV1 {
                code: "%%v0.1%%vA".into(),
            },
        }));
        let mut context = leader.presentation.clone();
        context.arrow_size = 9.;
        leader.annotation = Some(IfccadAnnotation {
            orientation: None,
            mode: IfccadAnnotationMode::Enabled,
            reference: None,
            contexts: vec![IfccadAnnotationContext {
                key: IfccadAnnotationContextKey {
                    scale_id: IfccadAnnotationScaleId(999),
                },
                presentation: Some(context),
            }],
        });
        leader.graphics = Some(IfccadLeaderGraphics {
            origin: IfccadLeaderGraphicsOrigin::SourceSnapshot,
            placement: leader.presentation.plane.clone(),
            commands: vec![IfccadLeaderGraphicCommand::CadCompatibility51Zero],
        });
        let raw = values(&leader);
        assert!(raw
            .iter()
            .any(|node| node["path"] == "/cad/d1/e80/context999/content/tolerance"));
        assert!(raw
            .iter()
            .any(|node| node["path"] == "/cad/d1/e80/graphics/op0"));
        assert!(raw
            .iter()
            .skip(1)
            .all(|node| node["attributes"].get("ifccad::entity").is_none()));
        let index = raw
            .iter()
            .map(|node| (node["path"].as_str().unwrap().to_owned(), node))
            .collect::<BTreeMap<_, _>>();
        let mut reader = Reader::new(&index, "/cad/d1");
        let back = reader.read_multi_leader("/cad/d1/e80").unwrap();
        reader.finish().unwrap();
        assert_eq!(leader, back);
    }
    #[test]
    fn owned_graph_rejects_orphans_aliases_and_inline_owned_payloads() {
        let mut raw = values(&IfccadMultiLeader::new());
        raw.push(json!({"path":"/cad/d1/e80/graphics/op99","attributes":{"ifccad::leaderGraphicsCommand":{"kind":"cadCompatibility51Zero"}}}));
        let index = raw
            .iter()
            .map(|node| (node["path"].as_str().unwrap().to_owned(), node))
            .collect::<BTreeMap<_, _>>();
        let mut reader = Reader::new(&index, "/cad/d1");
        reader.read_multi_leader("/cad/d1/e80").unwrap();
        assert!(reader.finish().is_err());
        let mut raw = values(&IfccadMultiLeader::new());
        raw[0]["children"]["leaders"] = json!("/cad/d1/e80/alias");
        let index = raw
            .iter()
            .map(|node| (node["path"].as_str().unwrap().to_owned(), node))
            .collect::<BTreeMap<_, _>>();
        assert!(Reader::new(&index, "/cad/d1")
            .read_multi_leader("/cad/d1/e80")
            .is_err());
        let mut raw = values(&IfccadMultiLeader::new());
        raw[0]["attributes"]["ifccad::multiLeader"]["presentation"]["groups"] = json!([]);
        let index = raw
            .iter()
            .map(|node| (node["path"].as_str().unwrap().to_owned(), node))
            .collect::<BTreeMap<_, _>>();
        assert!(Reader::new(&index, "/cad/d1")
            .read_multi_leader("/cad/d1/e80")
            .is_err());
    }
}
