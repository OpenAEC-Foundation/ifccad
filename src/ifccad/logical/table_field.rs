//! Stored evaluators and formula references for native table cell contents.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableFieldGraph {
    pub root: u64,
    pub next_id: u64,
    pub records: Vec<IfccadTableFieldRecord>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableFieldRecord {
    pub id: u64,
    pub evaluator: String,
    pub code: Vec<IfccadTableFieldCodePart>,
    pub children: Vec<u64>,
    pub object_targets: Vec<u64>,
    pub format: String,
    pub evaluation_options: i32,
    pub filing_options: i32,
    pub state: i32,
    pub evaluation_status: i32,
    pub error_code: i32,
    pub error_message: String,
    pub cache: IfccadTableFieldCache,
    pub auxiliary_values: Vec<IfccadTableFieldAuxiliaryValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableFieldCache {
    pub value: IfccadTableValue,
    pub display: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<IfccadTableUnit>,
    pub value_format: String,
    pub formatted_value: String,
    pub flags: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableFieldAuxiliaryValue {
    pub key: String,
    pub cache: IfccadTableFieldCache,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableCellReference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub table_id: Option<u64>,
    pub range: IfccadTableRange,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum IfccadTableFieldCodePart {
    Literal(String),
    Child(u64),
    Object(u64),
    Cells(IfccadTableCellReference),
}

impl IfccadTableFieldGraph {
    pub(super) fn validate_at(
        &self,
        document: &IfccadDocument,
        current: &IfccadTableGrid,
        path: &str,
    ) -> Result<(), IfccadReport> {
        let fail = |message| super::table::table_error(path, message);
        let records: BTreeMap<_, _> = self
            .records
            .iter()
            .map(|record| (record.id, record))
            .collect();
        if records.len() != self.records.len()
            || self.next_id == 0
            || records.keys().any(|id| *id >= self.next_id)
            || !records.contains_key(&self.root)
        {
            return Err(fail("invalid table field identities"));
        }
        let mut owners = BTreeMap::new();
        for record in &self.records {
            let mut children = BTreeSet::new();
            for child in &record.children {
                if *child == self.root
                    || !records.contains_key(child)
                    || !children.insert(*child)
                    || owners.insert(*child, record.id).is_some()
                {
                    return Err(fail("invalid table field child ownership"));
                }
            }
            let mut objects = BTreeSet::new();
            for id in &record.object_targets {
                if !objects.insert(*id) || super::table::native_entity(document, *id).is_none() {
                    return Err(fail("invalid table field object target"));
                }
            }
            for part in &record.code {
                match part {
                    IfccadTableFieldCodePart::Literal(code)
                        if super::attribute_fields::has_unbound_reference(code) =>
                    {
                        return Err(fail("table field literal contains an unbound reference"));
                    }
                    IfccadTableFieldCodePart::Child(id) if !children.contains(id) => {
                        return Err(fail("unbound table field child"));
                    }
                    IfccadTableFieldCodePart::Object(id) if !objects.contains(id) => {
                        return Err(fail("unbound table field object"));
                    }
                    IfccadTableFieldCodePart::Cells(reference) => {
                        let grid = match reference.table_id {
                            None => current,
                            Some(id) => match super::table::native_entity(document, id)
                                .map(|entity| &entity.kind)
                            {
                                Some(IfccadEntityKind::Table(table)) => &table.grid,
                                _ => return Err(fail("unresolved table formula target")),
                            },
                        };
                        if !reference.range.fits(grid.rows.len(), grid.columns.len()) {
                            return Err(fail("table formula range outside target grid"));
                        }
                    }
                    _ => {}
                }
            }
            for cache in std::iter::once(&record.cache)
                .chain(record.auxiliary_values.iter().map(|value| &value.cache))
            {
                cache.value.validate_at(document, path)?;
            }
        }
        super::attribute_fields::validate_owned_graph(
            self.root,
            &records,
            |record| record.children.iter().copied(),
            fail,
        )
    }
}
