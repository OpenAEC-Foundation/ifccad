//! Version-one stored CAD field expressions; opening never evaluates them.
use super::IfccadReport;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadAttributeFieldGraph {
    pub root: u64,
    pub next_id: u64,
    pub records: Vec<IfccadAttributeFieldRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IfccadAttributeFieldEvaluator {
    TextTemplate,
    DrawingVariable,
    ObjectProperty,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadAttributeFieldRecord {
    pub id: u64,
    pub evaluator: IfccadAttributeFieldEvaluator,
    pub code: Vec<IfccadAttributeFieldCodePart>,
    pub format: String,
    pub cache: IfccadAttributeFieldCache,
    pub evaluation_options: i32,
    pub filing_options: i32,
    pub state: i32,
    pub evaluation_status: i32,
    pub error_code: i32,
    pub error_message: String,
    #[serde(default)]
    pub source_evaluator: String,
    #[serde(default)]
    pub auxiliary_values: Vec<IfccadAttributeFieldAuxiliaryValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum IfccadAttributeFieldCodePart {
    Literal(String),
    Child(u64),
    Object(u64),
    CurrentBlockReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum IfccadAttributeFieldValue {
    Empty,
    Text(String),
    Integer(i64),
    Real(f64),
    Boolean(bool),
    Object(u64),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IfccadAttributeFieldCache {
    pub value: IfccadAttributeFieldValue,
    pub display: String,
    #[serde(default)]
    pub flags: i32,
    #[serde(default)]
    pub unit_type: i32,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub formatted_value: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IfccadAttributeFieldAuxiliaryValue {
    pub key: String,
    pub cache: IfccadAttributeFieldCache,
}

pub(super) fn has_unbound_reference(code: &str) -> bool {
    ["\\_ObjId", "\\_ObjIdx", "\\_FldIdx", "?BlockRefId"]
        .iter()
        .any(|key| code.contains(key))
}

impl IfccadAttributeFieldGraph {
    pub(super) fn validate(
        &self,
        entities: &BTreeSet<u64>,
        definition: bool,
        path: &str,
    ) -> Result<(), IfccadReport> {
        let fail = |m| super::attributes::fail(path, m);
        let records: BTreeMap<_, _> = self.records.iter().map(|r| (r.id, r)).collect();
        if records.len() != self.records.len()
            || self.next_id == 0
            || records.keys().any(|i| *i >= self.next_id)
            || !records.contains_key(&self.root)
        {
            return Err(fail("invalid attribute field identities/root/watermark"));
        }
        let mut owners = BTreeMap::new();
        for r in &self.records {
            for c in std::iter::once(&r.cache).chain(r.auxiliary_values.iter().map(|a| &a.cache)) {
                if let IfccadAttributeFieldValue::Object(id) = c.value {
                    if !entities.contains(&id) {
                        return Err(fail("unresolved field cache object"));
                    }
                }
            }
            if matches!(r.cache.value, IfccadAttributeFieldValue::Real(v) if !v.is_finite()) {
                return Err(fail("nonfinite field cache"));
            }
            if r.auxiliary_values.iter().any(
                |v| matches!(v.cache.value,IfccadAttributeFieldValue::Real(x) if !x.is_finite()),
            ) {
                return Err(fail("nonfinite auxiliary field cache"));
            }
            if !r.source_evaluator.is_empty()
                && !match r.evaluator {
                    IfccadAttributeFieldEvaluator::TextTemplate => r.source_evaluator == "_text",
                    IfccadAttributeFieldEvaluator::DrawingVariable => {
                        ["AcVar", "AcVar.16.2"].contains(&r.source_evaluator.as_str())
                    }
                    IfccadAttributeFieldEvaluator::ObjectProperty => {
                        ["AcObjProp", "AcObjProp.16.2"].contains(&r.source_evaluator.as_str())
                    }
                }
            {
                return Err(fail(
                    "source evaluator does not match selected field family",
                ));
            }
            for part in &r.code {
                match part {
                    IfccadAttributeFieldCodePart::Literal(s) if has_unbound_reference(s) => {
                        return Err(fail("field literal contains an unbound reference"))
                    }
                    IfccadAttributeFieldCodePart::Object(id) if !entities.contains(id) => {
                        return Err(fail("unresolved field object"))
                    }
                    IfccadAttributeFieldCodePart::CurrentBlockReference if !definition => {
                        return Err(fail(
                            "symbolic block reference requires an attribute definition",
                        ))
                    }
                    IfccadAttributeFieldCodePart::Child(id)
                        if !records.contains_key(id)
                            || *id == self.root
                            || owners.insert(*id, r.id).is_some_and(|owner| owner != r.id) =>
                    {
                        return Err(fail("invalid field child ownership"));
                    }
                    _ => {}
                }
            }
        }
        validate_owned_graph(
            self.root,
            &records,
            |record| {
                record.code.iter().filter_map(|part| match part {
                    IfccadAttributeFieldCodePart::Child(child) => Some(*child),
                    _ => None,
                })
            },
            fail,
        )
    }
}

pub(super) fn validate_owned_graph<'a, T, I: IntoIterator<Item = u64>>(
    root: u64,
    records: &BTreeMap<u64, &'a T>,
    children: impl Fn(&'a T) -> I,
    fail: impl Fn(&'static str) -> IfccadReport,
) -> Result<(), IfccadReport> {
    let mut seen = BTreeSet::new();
    let mut active = BTreeSet::new();
    let mut pending = vec![(root, false)];
    while let Some((id, exiting)) = pending.pop() {
        if exiting {
            active.remove(&id);
            seen.insert(id);
            continue;
        }
        if active.contains(&id) {
            return Err(fail("attribute field cycle"));
        }
        if seen.contains(&id) {
            continue;
        }
        active.insert(id);
        pending.push((id, true));
        for child in children(records[&id]) {
            pending.push((child, false));
        }
    }
    if seen.len() != records.len() {
        return Err(fail("unreachable attribute field record"));
    }
    Ok(())
}
