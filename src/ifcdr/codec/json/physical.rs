use super::mapping::{
    canonical_registry, Cardinality, FieldSchema, IfcdrRegistry, Presence, StreamRole, ValueType,
};
use crate::diagnostic::{
    PackageDiagnostic, PackageDiagnosticContextValue, PackageDiagnosticSeverity,
};
use crate::ifcdr::logical::IFCCAD_IFCDR_STRUCTURE_INVALID;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
const IFCCAD_IFCDR_DIRECTORY_INVALID: &str = "IFCCAD_IFCDR_DIRECTORY_INVALID";
const IFCCAD_IFCDR_STREAM_SCHEMA_UNSUPPORTED: &str = "IFCCAD_IFCDR_STREAM_SCHEMA_UNSUPPORTED";
#[derive(Debug)]
struct ValidatedTable {
    row_count: usize,
    ids: BTreeMap<u64, usize>,
}
#[derive(Debug)]
struct ValidatedStream {
    registry_index: usize,
    row_count: usize,
}

pub(super) fn validate_physical(uri: &str, value: &Value) -> Vec<PackageDiagnostic> {
    let mut validator = ResourceValidator::new(uri, canonical_registry());
    if let Some(version) = value.pointer("/header/version").and_then(Value::as_str) {
        if version != "0.7.0" {
            validator.error_with_context(
                "IFCCAD_IFCDR_VERSION_UNSUPPORTED",
                "/header/version",
                "unsupported IFCDR version",
                BTreeMap::from([
                    (
                        "actualVersion".into(),
                        PackageDiagnosticContextValue::String(version.into()),
                    ),
                    (
                        "supportedVersion".into(),
                        PackageDiagnosticContextValue::String("0.7.0".into()),
                    ),
                ]),
            );
            return validator.diagnostics;
        }
    }
    let Some(root) = value.as_object() else {
        validator.error(
            IFCCAD_IFCDR_STRUCTURE_INVALID,
            "",
            "IFCDR resource root must be an object",
        );
        return validator.diagnostics;
    };
    validator.validate_root_fields(root);
    if value.pointer("/header/format").and_then(Value::as_str) != Some("openaec.ifcdr") {
        validator.error(
            IFCCAD_IFCDR_STRUCTURE_INVALID,
            "/header/format",
            "invalid IFCDR format",
        );
    }
    validator.validate_tables(root);
    validator.validate_directory_and_streams(root);
    if validator.diagnostics.is_empty() {
        let order = &value["streams"]["entityOrderStream"];
        let mut offset = 0u64;
        for (row, start) in order["entryOffset"]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
        {
            if start.as_u64() != Some(offset) {
                validator.error(
                    "IFCCAD_IFCDR_ENTITY_ORDER_INVALID",
                    &format!("/streams/entityOrderStream/entryOffset/{row}"),
                    "order ranges must be contiguous from zero",
                );
            }
            offset += order["entryCount"][row].as_u64().expect("physical count");
        }
        let entries = value["streams"]["entityOrderEntryStream"]["count"]
            .as_u64()
            .unwrap_or(0);
        if offset != entries {
            validator.error(
                "IFCCAD_IFCDR_ENTITY_ORDER_INVALID",
                "/streams/entityOrderEntryStream/count",
                "order ranges must cover the complete entry stream",
            );
        }
    }
    validator.diagnostics
}

struct ResourceValidator<'a> {
    uri: &'a str,
    registry: &'a IfcdrRegistry,
    diagnostics: Vec<PackageDiagnostic>,
    has_unsupported_streams: bool,
}

impl<'a> ResourceValidator<'a> {
    fn new(uri: &'a str, registry: &'a IfcdrRegistry) -> Self {
        Self {
            uri,
            registry,
            diagnostics: Vec::new(),
            has_unsupported_streams: false,
        }
    }

    fn validate_root_fields(&mut self, root: &Map<String, Value>) {
        let mut allowed = self
            .registry
            .resource()
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<BTreeSet<_>>();
        for table in self.registry.tables() {
            if let Some(top) = table.payload_path().split('.').next() {
                allowed.insert(top);
            }
        }
        for key in root.keys() {
            if !allowed.contains(key.as_str()) {
                self.error(
                    IFCCAD_IFCDR_STRUCTURE_INVALID,
                    &pointer("", key),
                    "unknown IFCDR resource field",
                );
            }
        }
        for field in &self.registry.resource().fields {
            self.validate_field(root, field, "");
        }
    }

    fn validate_tables(&mut self, root: &Map<String, Value>) -> BTreeMap<String, ValidatedTable> {
        let mut evidence = BTreeMap::new();
        for table in self.registry.tables() {
            let value = value_at_path(root, table.payload_path());
            let Some(value) = value else {
                if table.presence == Presence::Required {
                    self.error(
                        IFCCAD_IFCDR_STRUCTURE_INVALID,
                        &path_pointer(table.payload_path()),
                        "required IFCDR table is missing",
                    );
                }
                continue;
            };
            let Some(rows) = value.as_array() else {
                self.error(
                    IFCCAD_IFCDR_STRUCTURE_INVALID,
                    &path_pointer(table.payload_path()),
                    "IFCDR table must be an array",
                );
                continue;
            };
            let ids = BTreeMap::new();
            for (row_index, row) in rows.iter().enumerate() {
                let row_pointer = format!("{}/{}", path_pointer(table.payload_path()), row_index);
                let Some(object) = row.as_object() else {
                    self.error(
                        IFCCAD_IFCDR_STRUCTURE_INVALID,
                        &row_pointer,
                        "IFCDR table row must be an object",
                    );
                    continue;
                };
                self.validate_closed_fields(object, &table.row_fields, &row_pointer);
            }
            evidence.insert(
                table.payload_path().to_owned(),
                ValidatedTable {
                    row_count: rows.len(),
                    ids: ids.clone(),
                },
            );
        }
        evidence
    }

    fn validate_directory_and_streams(
        &mut self,
        root: &Map<String, Value>,
    ) -> BTreeMap<String, ValidatedStream> {
        let Some(directory) = root.get("streamDirectory").and_then(Value::as_object) else {
            return BTreeMap::new();
        };
        self.validate_closed_fields(
            directory,
            &self.registry.directory().fields,
            "/streamDirectory",
        );
        if directory.get("version").and_then(Value::as_str)
            != Some(self.registry.directory().schema_id())
        {
            self.error(
                IFCCAD_IFCDR_DIRECTORY_INVALID,
                "/streamDirectory/version",
                "unsupported IFCDR stream directory",
            );
            return BTreeMap::new();
        }
        let Some(entries) = directory.get("streams").and_then(Value::as_array) else {
            return BTreeMap::new();
        };
        let Some(payloads) = root.get("streams").and_then(Value::as_object) else {
            return BTreeMap::new();
        };
        let directory_names = entries
            .iter()
            .filter_map(Value::as_object)
            .filter_map(|entry| entry.get("name"))
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>();
        let mut evidence = BTreeMap::new();
        let mut seen_names = BTreeSet::new();
        let mut claimed_payloads = BTreeSet::new();
        for (entry_index, entry) in entries.iter().enumerate() {
            let entry_pointer = format!("/streamDirectory/streams/{entry_index}");
            let Some(entry) = entry.as_object() else {
                self.error(
                    IFCCAD_IFCDR_DIRECTORY_INVALID,
                    &entry_pointer,
                    "stream directory entry must be an object",
                );
                continue;
            };
            self.validate_closed_fields(
                entry,
                &self.registry.directory().entry_fields,
                &entry_pointer,
            );
            let Some(name) = entry.get("name").and_then(Value::as_str) else {
                continue;
            };
            if !seen_names.insert(name) {
                self.error(
                    IFCCAD_IFCDR_DIRECTORY_INVALID,
                    &format!("{entry_pointer}/name"),
                    "stream directory names must be unique",
                );
                continue;
            }
            let Some(stream) = self.registry.stream_by_name(name) else {
                self.unsupported_stream(
                    name,
                    entry.get("schema").and_then(Value::as_str),
                    &entry_pointer,
                );
                continue;
            };
            let actual_schema = entry.get("schema").and_then(Value::as_str);
            if actual_schema != Some(stream.schema_id()) {
                if actual_schema
                    .and_then(|id| self.registry.stream_by_schema_id(id))
                    .is_none()
                {
                    self.unsupported_stream(name, actual_schema, &entry_pointer);
                } else {
                    self.error(
                        IFCCAD_IFCDR_DIRECTORY_INVALID,
                        &format!("{entry_pointer}/schema"),
                        "stream schema does not match its registered name",
                    );
                }
                continue;
            }
            if entry.get("role").and_then(Value::as_str) != Some(role_name(stream.role())) {
                self.error(
                    IFCCAD_IFCDR_DIRECTORY_INVALID,
                    &format!("{entry_pointer}/role"),
                    "stream role does not match the registry",
                );
            }
            let actual_parent = entry.get("parent").and_then(Value::as_str);
            if actual_parent != stream.parent.as_deref() {
                self.error(
                    IFCCAD_IFCDR_DIRECTORY_INVALID,
                    &format!("{entry_pointer}/parent"),
                    "stream parent does not match the registry",
                );
            }
            if stream
                .parent
                .as_deref()
                .is_some_and(|parent| !directory_names.contains(parent))
            {
                self.error(
                    IFCCAD_IFCDR_DIRECTORY_INVALID,
                    &format!("{entry_pointer}/parent"),
                    "stream parent is not present in the directory",
                );
            }
            let mut actual_children = Vec::new();
            let mut seen_children = BTreeSet::new();
            if let Some(items) = entry.get("children").and_then(Value::as_array) {
                for (index, item) in items.iter().enumerate() {
                    let Some(child) = item.as_str() else {
                        self.error(
                            IFCCAD_IFCDR_DIRECTORY_INVALID,
                            &format!("{entry_pointer}/children/{index}"),
                            "stream child name must be a string",
                        );
                        continue;
                    };
                    if !seen_children.insert(child) {
                        self.error(
                            IFCCAD_IFCDR_DIRECTORY_INVALID,
                            &format!("{entry_pointer}/children/{index}"),
                            "stream child names must be unique",
                        );
                        continue;
                    }
                    actual_children.push(child);
                }
            }
            let expected_children = stream
                .children
                .iter()
                .map(String::as_str)
                .filter(|child| directory_names.contains(child))
                .collect::<Vec<_>>();
            if actual_children != expected_children {
                self.error(
                    IFCCAD_IFCDR_DIRECTORY_INVALID,
                    &format!("{entry_pointer}/children"),
                    "stream children do not match the registry",
                );
            }
            let Some(count) = entry
                .get("count")
                .and_then(as_u64)
                .and_then(|v| usize::try_from(v).ok())
            else {
                continue;
            };
            let key = stream.payload_key();
            claimed_payloads.insert(key);
            let Some(payload) = payloads.get(key).and_then(Value::as_object) else {
                self.error(
                    IFCCAD_IFCDR_DIRECTORY_INVALID,
                    &format!("/streams/{}", escape(key)),
                    "registered stream payload is missing",
                );
                continue;
            };
            if payload.get("count").and_then(as_u64) != Some(count as u64) {
                self.error(
                    IFCCAD_IFCDR_DIRECTORY_INVALID,
                    &format!("/streams/{}/count", escape(key)),
                    "stream payload count differs from directory count",
                );
            }
            self.validate_stream_columns(root, entry, &entry_pointer, payload, stream, count);
            let registry_index = self
                .registry
                .streams()
                .iter()
                .position(|candidate| candidate.name() == name)
                .expect("registered stream index");
            evidence.insert(
                name.to_owned(),
                ValidatedStream {
                    registry_index,
                    row_count: count,
                },
            );
        }
        for key in payloads.keys() {
            if !self.has_unsupported_streams
                && !claimed_payloads.contains(key.as_str())
                && self
                    .registry
                    .table_by_payload_path(&format!("streams.{key}"))
                    .is_none()
            {
                self.error(
                    IFCCAD_IFCDR_DIRECTORY_INVALID,
                    &format!("/streams/{}", escape(key)),
                    "stream payload has no directory entry",
                );
            }
        }
        evidence
    }

    fn unsupported_stream(&mut self, name: &str, schema: Option<&str>, entry_pointer: &str) {
        self.has_unsupported_streams = true;
        let mut context = BTreeMap::from([(
            "streamName".to_owned(),
            PackageDiagnosticContextValue::String(name.to_owned()),
        )]);
        if let Some(schema) = schema {
            context.insert(
                "schemaId".to_owned(),
                PackageDiagnosticContextValue::String(schema.to_owned()),
            );
        }
        self.error_with_context(
            IFCCAD_IFCDR_STREAM_SCHEMA_UNSUPPORTED,
            &format!("{entry_pointer}/schema"),
            &format!(
                "stream schema is not supported by the IFCDR {} registry",
                self.registry.ifcdr_version()
            ),
            context,
        );
    }

    fn validate_stream_columns(
        &mut self,
        root: &Map<String, Value>,
        entry: &Map<String, Value>,
        entry_pointer: &str,
        payload: &Map<String, Value>,
        stream: &super::mapping::StreamSchema,
        count: usize,
    ) {
        let key = stream.payload_key();
        let mut listed = BTreeSet::new();
        if let Some(items) = entry.get("columns").and_then(Value::as_array) {
            for (index, item) in items.iter().enumerate() {
                let Some(column) = item.as_str() else {
                    self.error(
                        IFCCAD_IFCDR_DIRECTORY_INVALID,
                        &format!("{entry_pointer}/columns/{index}"),
                        "stream column name must be a string",
                    );
                    continue;
                };
                if !listed.insert(column) {
                    self.error(
                        IFCCAD_IFCDR_DIRECTORY_INVALID,
                        &format!("{entry_pointer}/columns/{index}"),
                        "stream column names must be unique",
                    );
                }
            }
        }
        let actual = payload
            .keys()
            .filter(|name| name.as_str() != "count")
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if listed != actual {
            self.error(
                IFCCAD_IFCDR_DIRECTORY_INVALID,
                &format!("/streams/{}/columns", escape(key)),
                "directory columns do not match physical payload columns",
            );
        }
        let allowed = stream
            .columns
            .iter()
            .map(|column| column.name.as_str())
            .collect::<BTreeSet<_>>();
        for name in &actual {
            if !allowed.contains(name) {
                self.error(
                    IFCCAD_IFCDR_STRUCTURE_INVALID,
                    &format!("/streams/{}/{}", escape(key), escape(name)),
                    "unknown stream column",
                );
            }
        }
        for column in &stream.columns {
            let pointer = format!("/streams/{}/{}", escape(key), escape(&column.name));
            let Some(value) = payload.get(&column.name) else {
                if column.presence == Presence::Required {
                    self.error(
                        IFCCAD_IFCDR_STRUCTURE_INVALID,
                        &pointer,
                        "required stream column is missing",
                    );
                }
                continue;
            };
            let Some(values) = value.as_array() else {
                self.error(
                    IFCCAD_IFCDR_STRUCTURE_INVALID,
                    &pointer,
                    "stream column must be an array",
                );
                continue;
            };
            if column.cardinality == Cardinality::Row && values.len() != count {
                self.error(
                    IFCCAD_IFCDR_STRUCTURE_INVALID,
                    &pointer,
                    "row column length must equal stream count",
                );
            }
            for (index, item) in values.iter().enumerate() {
                if !valid_scalar(item, column.value_type, column.nullable) {
                    self.error(
                        IFCCAD_IFCDR_STRUCTURE_INVALID,
                        &format!("{pointer}/{index}"),
                        "stream column value has the wrong physical type",
                    );
                }
            }
        }
        let mut pool_length = None;
        for column in stream
            .columns
            .iter()
            .filter(|column| column.cardinality == Cardinality::Pool)
        {
            let Some(length) = payload
                .get(&column.name)
                .and_then(Value::as_array)
                .map(Vec::len)
            else {
                continue;
            };
            match pool_length {
                None => pool_length = Some(length),
                Some(expected) if expected != length => self.error(
                    IFCCAD_IFCDR_STRUCTURE_INVALID,
                    &format!("/streams/{}/{}", escape(key), escape(&column.name)),
                    "pool columns in one stream must have synchronized lengths",
                ),
                Some(_) => {}
            }
        }
        for range in &stream.ranges {
            let offsets = payload.get(&range.offset_column).and_then(Value::as_array);
            let counts = payload.get(&range.count_column).and_then(Value::as_array);
            let Some((offsets, counts)) = offsets.zip(counts) else {
                continue;
            };
            for row in 0..offsets.len().min(counts.len()) {
                let Some((offset, range_count)) = offsets[row].as_u64().zip(counts[row].as_u64())
                else {
                    continue;
                };
                let Some(end) = offset.checked_add(range_count) else {
                    self.error(
                        IFCCAD_IFCDR_STRUCTURE_INVALID,
                        &format!(
                            "/streams/{}/{}/{row}",
                            escape(key),
                            escape(&range.count_column)
                        ),
                        "stream range overflows",
                    );
                    continue;
                };
                for target in &range.target_columns {
                    if payload
                        .get(target)
                        .and_then(Value::as_array)
                        .is_some_and(|values| end > values.len() as u64)
                    {
                        self.error(
                            IFCCAD_IFCDR_STRUCTURE_INVALID,
                            &format!(
                                "/streams/{}/{}/{row}",
                                escape(key),
                                escape(&range.count_column)
                            ),
                            "stream range exceeds its target column",
                        );
                    }
                }
                if let Some(target_path) = &range.target_path {
                    let target_len = value_at_path(root, target_path).and_then(|target| {
                        target.as_array().map(Vec::len).or_else(|| {
                            target
                                .as_object()
                                .and_then(|object| object.get("count"))
                                .and_then(Value::as_u64)
                                .and_then(|count| usize::try_from(count).ok())
                        })
                    });
                    let location = format!(
                        "/streams/{}/{}/{row}",
                        escape(key),
                        escape(&range.count_column)
                    );
                    match target_len {
                        Some(length) if end > length as u64 => self.error(
                            IFCCAD_IFCDR_STRUCTURE_INVALID,
                            &location,
                            "stream range exceeds its external target payload",
                        ),
                        None if range_count > 0 => self.error(
                            IFCCAD_IFCDR_STRUCTURE_INVALID,
                            &location,
                            "nonempty stream range has no external target payload",
                        ),
                        _ => {}
                    }
                }
            }
        }
    }

    fn validate_field(&mut self, object: &Map<String, Value>, field: &FieldSchema, base: &str) {
        let location = pointer(base, &field.name);
        let Some(value) = object.get(&field.name) else {
            if field.presence == Presence::Required {
                self.error(
                    IFCCAD_IFCDR_STRUCTURE_INVALID,
                    &location,
                    "required IFCDR field is missing",
                );
            }
            return;
        };
        if !valid_scalar(value, field.value_type, field.nullable) {
            self.error(
                IFCCAD_IFCDR_STRUCTURE_INVALID,
                &location,
                "IFCDR field has the wrong physical type",
            );
            return;
        }
        if !field.fields.is_empty() {
            if let Some(nested) = value.as_object() {
                self.validate_closed_fields(nested, &field.fields, &location);
            }
        }
    }

    fn validate_closed_fields(
        &mut self,
        object: &Map<String, Value>,
        fields: &[FieldSchema],
        base: &str,
    ) {
        let allowed = fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<BTreeSet<_>>();
        for key in object.keys() {
            if !allowed.contains(key.as_str()) {
                self.error(
                    IFCCAD_IFCDR_STRUCTURE_INVALID,
                    &pointer(base, key),
                    "unknown field in closed IFCDR object",
                );
            }
        }
        for field in fields {
            self.validate_field(object, field, base);
        }
    }

    fn error(&mut self, code: &str, location: &str, message: &str) {
        self.error_with_context(code, location, message, BTreeMap::new());
    }

    fn error_with_context(
        &mut self,
        code: &str,
        location: &str,
        message: &str,
        context: BTreeMap<String, PackageDiagnosticContextValue>,
    ) {
        self.diagnostics.push(PackageDiagnostic {
            category: match code {
                "IFCCAD_IFCDR_VERSION_UNSUPPORTED" | IFCCAD_IFCDR_STREAM_SCHEMA_UNSUPPORTED => {
                    crate::diagnostic::PackageDiagnosticCategory::UnsupportedContent
                }
                IFCCAD_IFCDR_STRUCTURE_INVALID
                | IFCCAD_IFCDR_DIRECTORY_INVALID
                | "IFCCAD_IFCDR_ENTITY_ORDER_INVALID" => {
                    crate::diagnostic::PackageDiagnosticCategory::ContractViolation
                }
                _ => unreachable!("unclassified physical diagnostic: {code}"),
            },
            code: code.to_owned(),
            severity: PackageDiagnosticSeverity::Error,
            resource_id: None,
            resource_uri: Some(self.uri.to_owned()),
            location: Some(location.to_owned()),
            context,
            message: message.to_owned(),
        });
    }
}

fn valid_scalar(value: &Value, kind: ValueType, nullable: bool) -> bool {
    if value.is_null() {
        return nullable;
    }
    match kind {
        ValueType::Array => value.is_array(),
        ValueType::Boolean => value.is_boolean(),
        ValueType::Float64 => finite_f64(value).is_some(),
        ValueType::IfcxId | ValueType::String => value.is_string(),
        ValueType::Int32 => value
            .as_i64()
            .is_some_and(|item| i32::try_from(item).is_ok()),
        ValueType::JsonValue => true,
        ValueType::Object => value.is_object(),
        ValueType::Uint32 => value
            .as_u64()
            .is_some_and(|item| u32::try_from(item).is_ok()),
        ValueType::Uint64 => value.as_u64().is_some(),
    }
}

fn as_u64(value: &Value) -> Option<u64> {
    value.as_u64()
}

fn finite_f64(value: &Value) -> Option<f64> {
    value.as_f64().filter(|number| number.is_finite())
}

fn value_at_path<'a>(root: &'a Map<String, Value>, path: &str) -> Option<&'a Value> {
    let mut value = root.get(path.split('.').next()?)?;
    for segment in path.split('.').skip(1) {
        value = value.as_object()?.get(segment)?;
    }
    Some(value)
}

fn role_name(role: StreamRole) -> &'static str {
    match role {
        StreamRole::Child => "child",
        StreamRole::Object => "object",
        StreamRole::Order => "order",
    }
}

fn path_pointer(path: &str) -> String {
    format!(
        "/{}",
        path.split('.').map(escape).collect::<Vec<_>>().join("/")
    )
}

fn pointer(base: &str, property: &str) -> String {
    format!("{base}/{}", escape(property))
}

fn escape(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}
