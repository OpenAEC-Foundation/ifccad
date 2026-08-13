use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

const REGISTRY_META_SCHEMA: &str = include_str!("../../schemas/ifcdr/registry-meta-schema-v1.json");
const CANONICAL_REGISTRY: &str = include_str!("../../schemas/ifcdr/registry-0.5.0.json");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IfcdrRegistry {
    #[serde(rename = "$schema")]
    schema: String,
    registry_schema_version: String,
    ifcdr_version: String,
    resource: ObjectSchema,
    directory: DirectorySchema,
    tables: Vec<TableSchema>,
    streams: Vec<StreamSchema>,
    #[serde(skip)]
    table_by_payload_path: BTreeMap<String, usize>,
    #[serde(skip)]
    stream_by_name: BTreeMap<String, usize>,
    #[serde(skip)]
    stream_by_schema_id: BTreeMap<String, usize>,
    #[serde(skip)]
    stream_by_payload_key: BTreeMap<String, usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObjectSchema {
    pub(crate) schema_id: String,
    pub(crate) fields: Vec<FieldSchema>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DirectorySchema {
    schema_id: String,
    pub(crate) fields: Vec<FieldSchema>,
    pub(crate) entry_fields: Vec<FieldSchema>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FieldSchema {
    pub(crate) name: String,
    pub(crate) value_type: ValueType,
    pub(crate) presence: Presence,
    pub(crate) nullable: bool,
    #[serde(default)]
    pub(crate) fields: Vec<FieldSchema>,
    #[serde(default)]
    pub(crate) omission_default: Option<Value>,
    #[serde(default)]
    pub(crate) reference: Option<Reference>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TableSchema {
    name: String,
    schema_id: String,
    payload_path: String,
    pub(crate) presence: Presence,
    pub(crate) row_fields: Vec<FieldSchema>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StreamSchema {
    name: String,
    schema_id: String,
    payload_key: String,
    role: StreamRole,
    pub(crate) columns: Vec<ColumnSchema>,
    #[serde(default)]
    pub(crate) ranges: Vec<RangeSchema>,
    #[serde(default)]
    pub(crate) parent: Option<String>,
    #[serde(default)]
    pub(crate) children: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ColumnSchema {
    pub(crate) name: String,
    pub(crate) value_type: ValueType,
    pub(crate) presence: Presence,
    pub(crate) nullable: bool,
    pub(crate) cardinality: Cardinality,
    #[serde(default)]
    pub(crate) omission_default: Option<Value>,
    #[serde(default)]
    pub(crate) reference: Option<Reference>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Reference {
    pub(crate) category: ReferenceCategory,
    pub(crate) target: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RangeSchema {
    pub(crate) offset_column: String,
    pub(crate) count_column: String,
    #[serde(default)]
    pub(crate) target_columns: Vec<String>,
    #[serde(default)]
    pub(crate) target_path: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ValueType {
    Array,
    Boolean,
    Float64,
    IfcxId,
    Int32,
    JsonValue,
    Object,
    String,
    Uint32,
    Uint64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Presence {
    Optional,
    Required,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Cardinality {
    Pool,
    Row,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum StreamRole {
    Child,
    Object,
    Order,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ReferenceCategory {
    Entity,
    Ifcx,
    StreamColumn,
    TableField,
}

impl IfcdrRegistry {
    fn build_indexes(&mut self) {
        self.table_by_payload_path = self
            .tables
            .iter()
            .enumerate()
            .map(|(index, table)| (table.payload_path.clone(), index))
            .collect();
        self.stream_by_name = self
            .streams
            .iter()
            .enumerate()
            .map(|(index, stream)| (stream.name.clone(), index))
            .collect();
        self.stream_by_schema_id = self
            .streams
            .iter()
            .enumerate()
            .map(|(index, stream)| (stream.schema_id.clone(), index))
            .collect();
        self.stream_by_payload_key = self
            .streams
            .iter()
            .enumerate()
            .map(|(index, stream)| (stream.payload_key.clone(), index))
            .collect();
    }

    pub(crate) fn ifcdr_version(&self) -> &str {
        &self.ifcdr_version
    }

    pub(crate) fn resource(&self) -> &ObjectSchema {
        &self.resource
    }

    pub(crate) fn directory(&self) -> &DirectorySchema {
        &self.directory
    }

    pub(crate) fn tables(&self) -> &[TableSchema] {
        &self.tables
    }

    pub(crate) fn streams(&self) -> &[StreamSchema] {
        &self.streams
    }

    pub(crate) fn table_by_payload_path(&self, path: &str) -> Option<&TableSchema> {
        self.table_by_payload_path
            .get(path)
            .map(|index| &self.tables[*index])
    }

    pub(crate) fn stream_by_name(&self, name: &str) -> Option<&StreamSchema> {
        self.stream_by_name
            .get(name)
            .map(|index| &self.streams[*index])
    }

    pub(crate) fn stream_by_schema_id(&self, id: &str) -> Option<&StreamSchema> {
        self.stream_by_schema_id
            .get(id)
            .map(|index| &self.streams[*index])
    }

    pub(crate) fn stream_by_payload_key(&self, key: &str) -> Option<&StreamSchema> {
        self.stream_by_payload_key
            .get(key)
            .map(|index| &self.streams[*index])
    }
}

impl DirectorySchema {
    pub(crate) fn schema_id(&self) -> &str {
        &self.schema_id
    }
}

impl TableSchema {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn schema_id(&self) -> &str {
        &self.schema_id
    }

    pub(crate) fn payload_path(&self) -> &str {
        &self.payload_path
    }
}

impl StreamSchema {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn schema_id(&self) -> &str {
        &self.schema_id
    }

    pub(crate) fn payload_key(&self) -> &str {
        &self.payload_key
    }

    pub(crate) fn role(&self) -> StreamRole {
        self.role
    }
}

pub(crate) fn canonical_registry() -> &'static IfcdrRegistry {
    static REGISTRY: OnceLock<IfcdrRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let meta_schema: Value = serde_json::from_str(REGISTRY_META_SCHEMA)
            .unwrap_or_else(|error| panic!("parse embedded IFCDR registry meta-schema: {error}"));
        let registry_value: Value = serde_json::from_str(CANONICAL_REGISTRY)
            .unwrap_or_else(|error| panic!("parse embedded IFCDR registry: {error}"));
        let validator = jsonschema::draft202012::new(&meta_schema)
            .unwrap_or_else(|error| panic!("compile embedded IFCDR registry meta-schema: {error}"));
        let errors = validator
            .iter_errors(&registry_value)
            .map(|error| format!("{}: {}", error.instance_path(), error.masked()))
            .collect::<Vec<_>>();
        assert!(
            errors.is_empty(),
            "embedded IFCDR registry violates its meta-schema: {}",
            errors.join("; ")
        );

        let mut registry: IfcdrRegistry = serde_json::from_value(registry_value)
            .unwrap_or_else(|error| panic!("deserialize embedded IFCDR registry: {error}"));
        registry.build_indexes();
        let errors = validate_registry_cross_references(&registry);
        assert!(
            errors.is_empty(),
            "embedded IFCDR registry has invalid references: {}",
            errors.join("; ")
        );
        registry
    })
}

fn validate_registry_cross_references(registry: &IfcdrRegistry) -> Vec<String> {
    let mut errors = Vec::new();
    check_unique(
        registry.tables.iter().map(|table| table.name.as_str()),
        "table name",
        &mut errors,
    );
    check_unique(
        registry.tables.iter().map(|table| table.schema_id.as_str()),
        "table schema ID",
        &mut errors,
    );
    check_unique(
        registry
            .tables
            .iter()
            .map(|table| table.payload_path.as_str()),
        "table payload path",
        &mut errors,
    );
    check_unique(
        registry.streams.iter().map(|stream| stream.name.as_str()),
        "stream name",
        &mut errors,
    );
    check_unique(
        registry
            .streams
            .iter()
            .map(|stream| stream.schema_id.as_str()),
        "stream schema ID",
        &mut errors,
    );
    check_unique(
        registry
            .streams
            .iter()
            .map(|stream| stream.payload_key.as_str()),
        "stream payload key",
        &mut errors,
    );

    for stream in &registry.streams {
        let columns = stream
            .columns
            .iter()
            .map(|column| column.name.as_str())
            .collect::<BTreeSet<_>>();
        if columns.len() != stream.columns.len() {
            errors.push(format!("stream {:?} has duplicate columns", stream.name));
        }
        for range in &stream.ranges {
            for column in [&range.offset_column, &range.count_column] {
                if !columns.contains(column.as_str()) {
                    errors.push(format!(
                        "stream {:?} range refers to missing column {:?}",
                        stream.name, column
                    ));
                }
            }
            for target in &range.target_columns {
                if !columns.contains(target.as_str()) {
                    errors.push(format!(
                        "stream {:?} range refers to missing target column {:?}",
                        stream.name, target
                    ));
                }
            }
            if let Some(path) = &range.target_path {
                let key = path.strip_prefix("streams.").unwrap_or(path);
                if registry.table_by_payload_path(path).is_none()
                    && registry.stream_by_payload_key(key).is_none()
                {
                    errors.push(format!(
                        "stream {:?} range refers to missing target path {:?}",
                        stream.name, path
                    ));
                }
            }
        }
        if let Some(parent) = &stream.parent {
            match registry.stream_by_name(parent) {
                Some(parent_stream) if parent_stream.children.contains(&stream.name) => {}
                Some(_) => errors.push(format!(
                    "stream {:?} is not listed by parent {:?}",
                    stream.name, parent
                )),
                None => errors.push(format!(
                    "stream {:?} has missing parent {:?}",
                    stream.name, parent
                )),
            }
        }
        for child in &stream.children {
            match registry.stream_by_name(child) {
                Some(child_stream) if child_stream.parent.as_deref() == Some(&stream.name) => {}
                Some(_) => errors.push(format!(
                    "stream {:?} child {:?} does not point back",
                    stream.name, child
                )),
                None => errors.push(format!(
                    "stream {:?} has missing child {:?}",
                    stream.name, child
                )),
            }
        }
        for column in &stream.columns {
            if let Some(reference) = &column.reference {
                validate_reference(registry, reference, &stream.name, &mut errors);
            }
        }
    }
    for table in &registry.tables {
        let fields = table
            .row_fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<BTreeSet<_>>();
        if fields.len() != table.row_fields.len() {
            errors.push(format!("table {:?} has duplicate row fields", table.name));
        }
        for field in &table.row_fields {
            if let Some(reference) = &field.reference {
                validate_reference(registry, reference, &table.name, &mut errors);
            }
        }
    }
    errors
}

fn validate_reference(
    registry: &IfcdrRegistry,
    reference: &Reference,
    owner: &str,
    errors: &mut Vec<String>,
) {
    let valid = match reference.category {
        ReferenceCategory::Entity => reference.target == "entity",
        ReferenceCategory::Ifcx => reference.target == "ifcx",
        ReferenceCategory::TableField => {
            reference
                .target
                .split_once('.')
                .is_some_and(|(payload, field)| {
                    registry
                        .table_by_payload_path(payload)
                        .is_some_and(|table| table.row_fields.iter().any(|item| item.name == field))
                })
        }
        ReferenceCategory::StreamColumn => {
            reference
                .target
                .split_once('.')
                .is_some_and(|(payload, column)| {
                    registry
                        .stream_by_payload_key(payload)
                        .is_some_and(|stream| stream.columns.iter().any(|item| item.name == column))
                })
        }
    };
    if !valid {
        errors.push(format!(
            "{owner:?} has invalid {:?} reference target {:?}",
            reference.category, reference.target
        ));
    }
}

fn check_unique<'a>(values: impl Iterator<Item = &'a str>, label: &str, errors: &mut Vec<String>) {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            errors.push(format!("duplicate {label} {value:?}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_registry_is_valid_complete_and_cached() {
        let first = canonical_registry();
        let second = canonical_registry();

        assert!(std::ptr::eq(first, second));
        assert_eq!(first.ifcdr_version(), "0.5.0");
        assert_eq!(
            first.directory().schema_id(),
            "ifccad.ifcdr.streamDirectory.v1"
        );
        assert_eq!(first.tables().len(), 12);
        assert_eq!(first.streams().len(), 29);

        let line = first.stream_by_name("line").expect("line registry entry");
        assert_eq!(line.schema_id(), "ifccad.ifcdr.line.v2");
        assert_eq!(line.payload_key(), "lineStream");
        assert_eq!(line.role(), StreamRole::Object);
        assert!(std::ptr::eq(
            line,
            first.stream_by_schema_id("ifccad.ifcdr.line.v2").unwrap()
        ));
        assert!(std::ptr::eq(
            line,
            first.stream_by_payload_key("lineStream").unwrap()
        ));
        assert_eq!(
            first
                .table_by_payload_path("appearanceBindings")
                .unwrap()
                .name(),
            "appearanceBinding"
        );
    }

    #[test]
    fn canonical_registry_cross_references_are_complete() {
        assert!(validate_registry_cross_references(canonical_registry()).is_empty());
    }
}
