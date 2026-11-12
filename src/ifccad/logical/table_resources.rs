//! Shared linked sources and standalone column-oriented data tables.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IfccadTableDataLinkId(pub u64);

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadTableDataLink {
    pub id: IfccadTableDataLinkId,
    pub settings: IfccadTableDataLinkSettings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IfccadDataTableId(pub u64);

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadDataTable {
    pub id: IfccadDataTableId,
    pub settings: IfccadDataTableSettings,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableDataLinkSettings {
    pub name: String,
    pub adapter: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_selection: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
    pub options: i32,
    pub update_options: i32,
    pub path_option: i16,
    pub status_flags: i32,
    pub flags: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_update: Option<[i16; 7]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update_status: Option<String>,
    pub custom_data: Vec<IfccadTableCustomValue>,
    pub custom_targets: Vec<IfccadTableDataLinkTarget>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableDataLinkTarget {
    pub object_id: u64,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableDataLinkRange {
    pub link_id: IfccadTableDataLinkId,
    pub range: IfccadTableRange,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableCustomValue {
    pub name: String,
    pub value: IfccadTableValue,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadDataTableSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<IfccadDataTableOwner>,
    pub name: String,
    pub flags: i16,
    pub row_count: usize,
    pub columns: Vec<IfccadDataTableColumn>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadDataTableOwner {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<u64>,
    pub dictionary_path: Vec<String>,
    pub key: String,
    pub hard_owned: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadDataTableColumn {
    pub name: String,
    pub value_type: IfccadDataTableColumnType,
    pub values: Vec<IfccadTableValue>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadDataTableColumnType {
    Integer,
    Real,
    Text,
    Point,
    Object,
    HardOwner,
    SoftOwner,
    HardPointer,
    SoftPointer,
    Boolean,
    Vector,
}

pub(super) fn validate_resources(document: &IfccadDocument) -> Result<(), IfccadReport> {
    use super::table::{native_entity, table_error};
    use std::collections::BTreeSet;
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for link in &document.table_data_links {
        let path = format!("/cad/d{}/tableDataLink/{}", document.drawing_id, link.id.0);
        let settings = &link.settings;
        if link.id.0 == 0
            || !ids.insert(link.id)
            || settings.name.trim().is_empty()
            || !names.insert(settings.name.to_lowercase())
        {
            return Err(table_error(
                &path,
                "invalid or duplicate table datalink ID or name",
            ));
        }
        for item in &settings.custom_data {
            item.value.validate_at(document, &path)?;
        }
        for target in &settings.custom_targets {
            if native_entity(document, target.object_id).is_none() {
                return Err(table_error(&path, "unresolved table datalink object"));
            }
        }
    }
    let mut ids = BTreeSet::new();
    for data in &document.data_tables {
        let path = format!("/cad/d{}/dataTable/{}", document.drawing_id, data.id.0);
        if data.id.0 == 0 || !ids.insert(data.id) {
            return Err(table_error(&path, "invalid or duplicate data table ID"));
        }
        if let Some(owner) = &data.settings.owner {
            if owner.key.is_empty() || owner.dictionary_path.iter().any(String::is_empty) {
                return Err(table_error(&path, "empty data table dictionary key"));
            }
            if owner
                .entity_id
                .is_some_and(|id| native_entity(document, id).is_none())
            {
                return Err(table_error(&path, "unresolved data table owner entity"));
            }
        }
        for column in &data.settings.columns {
            if column.values.len() != data.settings.row_count {
                return Err(table_error(
                    &path,
                    "data table column length differs from row count",
                ));
            }
            for value in &column.values {
                use IfccadDataTableColumnType as Column;
                use IfccadTableValue as Value;
                let matches_type = matches!(
                    (column.value_type, value),
                    (Column::Integer, Value::Integer(_))
                        | (Column::Real, Value::Real(_))
                        | (Column::Text, Value::Text(_))
                        | (Column::Point | Column::Vector, Value::Point3D(_))
                        | (Column::Boolean, Value::Boolean(_))
                        | (
                            Column::Object
                                | Column::HardOwner
                                | Column::SoftOwner
                                | Column::HardPointer
                                | Column::SoftPointer,
                            Value::Object(_)
                        )
                );
                if !matches_type {
                    return Err(table_error(
                        &path,
                        "data table value does not match its column type",
                    ));
                }
                value.validate_at(document, &path)?;
            }
        }
    }
    Ok(())
}
