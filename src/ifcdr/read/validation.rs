use super::codes::{
    IFCCAD_IFCDR_DIRECTORY_INVALID, IFCCAD_IFCDR_REFERENCE_MISSING,
    IFCCAD_IFCDR_STREAM_SCHEMA_UNSUPPORTED, IFCCAD_IFCDR_STRUCTURE_INVALID,
    IFCCAD_IFCDR_UNIT_UNSUPPORTED, IFCCAD_IFCDR_VERSION_UNSUPPORTED,
};
use super::registry::{
    canonical_registry, Cardinality, FieldSchema, IfcdrRegistry, Presence, ReferenceCategory,
    StreamRole, ValueType,
};
use super::resource::{
    IfcdrHeader, IfcdrValidationEvidence, LoadedIfcdrResource, ValidatedStream, ValidatedTable,
};
use crate::diagnostic::{
    PackageDiagnostic, PackageDiagnosticContextValue, PackageDiagnosticSeverity,
};
use crate::ifcdr::{Bounds2d, Point2};
use crate::validated::{EvidenceOutcome, Validated, ValidationOutcome};
use crate::ResourceId;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
#[cfg(test)]
use std::sync::Arc;

pub(crate) fn validate_ifcdr(
    loaded: LoadedIfcdrResource,
) -> ValidationOutcome<LoadedIfcdrResource> {
    Validated::validate(loaded, canonical_registry())
}

pub(super) fn build_evidence(
    loaded: &LoadedIfcdrResource,
    registry: &IfcdrRegistry,
) -> EvidenceOutcome<IfcdrValidationEvidence, PackageDiagnostic> {
    let mut validator = ResourceValidator::new(loaded.uri(), registry);
    let Some(root) = loaded.source().value().as_object() else {
        validator.error(
            IFCCAD_IFCDR_STRUCTURE_INVALID,
            "",
            "IFCDR resource root must be an object",
        );
        return EvidenceOutcome::failure(validator.diagnostics);
    };

    let version = root
        .get("header")
        .and_then(Value::as_object)
        .and_then(|header| header.get("version"))
        .and_then(Value::as_str);
    if let Some(version) = version {
        if version != registry.ifcdr_version() {
            validator.error_with_context(
                IFCCAD_IFCDR_VERSION_UNSUPPORTED,
                "/header/version",
                "unsupported IFCDR version",
                BTreeMap::from([
                    (
                        "actualVersion".to_owned(),
                        PackageDiagnosticContextValue::String(version.to_owned()),
                    ),
                    (
                        "supportedVersion".to_owned(),
                        PackageDiagnosticContextValue::String(registry.ifcdr_version().to_owned()),
                    ),
                ]),
            );
            return EvidenceOutcome::failure(validator.diagnostics);
        }
    }

    validator.validate_root_fields(root);
    let header = validator.validate_header(root);
    let bounds = validator.validate_bounds(root);
    let tables = validator.validate_tables(root);
    let streams = validator.validate_directory_and_streams(root);
    let entities = header.as_ref().map(|header| {
        super::entity::validate_entities(
            loaded.uri(),
            root,
            registry,
            &streams,
            header.next_entity_id,
        )
    });
    if let Some((_, entity_diagnostics)) = &entities {
        validator
            .diagnostics
            .extend(entity_diagnostics.iter().cloned());
    }
    if let Some((entity_index, _)) = &entities {
        validator.validate_registered_references(root, entity_index);
    }

    match (header, bounds, validator.diagnostics.is_empty()) {
        (Some(header), Some(bounds), true) => EvidenceOutcome::success(
            IfcdrValidationEvidence {
                header,
                bounds,
                tables,
                streams,
                entities: entities.expect("header exists with successful evidence").0,
            },
            Vec::new(),
        ),
        _ => EvidenceOutcome::failure(validator.diagnostics),
    }
}

struct ResourceValidator<'a> {
    uri: &'a str,
    registry: &'a IfcdrRegistry,
    diagnostics: Vec<PackageDiagnostic>,
}

impl<'a> ResourceValidator<'a> {
    fn new(uri: &'a str, registry: &'a IfcdrRegistry) -> Self {
        Self {
            uri,
            registry,
            diagnostics: Vec::new(),
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

    fn validate_header(&mut self, root: &Map<String, Value>) -> Option<IfcdrHeader> {
        let header = root.get("header")?.as_object()?;
        let format = header.get("format")?.as_str()?.to_owned();
        if format != "openaec.ifcdr" {
            self.error(
                IFCCAD_IFCDR_STRUCTURE_INVALID,
                "/header/format",
                "IFCDR header format must be openaec.ifcdr",
            );
        }
        let unit = header.get("unit")?.as_str()?.to_owned();
        if !matches!(
            unit.as_str(),
            "unitless" | "mm" | "cm" | "m" | "km" | "in" | "ft"
        ) {
            self.error_with_context(
                IFCCAD_IFCDR_UNIT_UNSUPPORTED,
                "/header/unit",
                "unsupported IFCDR length unit",
                BTreeMap::from([(
                    "actualUnit".to_owned(),
                    PackageDiagnosticContextValue::String(unit.clone()),
                )]),
            );
        }
        Some(IfcdrHeader {
            format,
            version: header.get("version")?.as_str()?.to_owned(),
            resource_id: ResourceId::new(header.get("resourceId")?.as_str()?).ok()?,
            unit,
            next_entity_id: as_u64(header.get("nextEntityId")?)?,
        })
    }

    fn validate_bounds(&self, root: &Map<String, Value>) -> Option<Bounds2d> {
        let bounds = root.get("bounds")?.as_object()?;
        Some(Bounds2d {
            min: Point2 {
                x: finite_f64(bounds.get("minX")?)?,
                y: finite_f64(bounds.get("minY")?)?,
            },
            max: Point2 {
                x: finite_f64(bounds.get("maxX")?)?,
                y: finite_f64(bounds.get("maxY")?)?,
            },
        })
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
            let mut ids = BTreeMap::new();
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
                if let Some(id) = object.get("id").and_then(as_u64) {
                    if let Some(first) = ids.insert(id, row_index) {
                        self.error_with_context(
                            IFCCAD_IFCDR_STRUCTURE_INVALID,
                            &format!("{row_pointer}/id"),
                            "IFCDR table ID must be unique",
                            BTreeMap::from([
                                (
                                    "id".to_owned(),
                                    PackageDiagnosticContextValue::Number(id.into()),
                                ),
                                (
                                    "firstRow".to_owned(),
                                    PackageDiagnosticContextValue::Number((first as u64).into()),
                                ),
                            ]),
                        );
                    }
                }
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
                self.error(
                    IFCCAD_IFCDR_STREAM_SCHEMA_UNSUPPORTED,
                    &format!("{entry_pointer}/schema"),
                    "stream schema is not registered for IFCDR 0.5.0",
                );
                continue;
            };
            let actual_schema = entry.get("schema").and_then(Value::as_str);
            if actual_schema != Some(stream.schema_id()) {
                let code = if actual_schema
                    .and_then(|id| self.registry.stream_by_schema_id(id))
                    .is_none()
                {
                    IFCCAD_IFCDR_STREAM_SCHEMA_UNSUPPORTED
                } else {
                    IFCCAD_IFCDR_DIRECTORY_INVALID
                };
                self.error(
                    code,
                    &format!("{entry_pointer}/schema"),
                    "stream schema does not match its registered name",
                );
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
            if !claimed_payloads.contains(key.as_str())
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

    fn validate_stream_columns(
        &mut self,
        root: &Map<String, Value>,
        entry: &Map<String, Value>,
        entry_pointer: &str,
        payload: &Map<String, Value>,
        stream: &super::registry::StreamSchema,
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

    fn validate_registered_references(
        &mut self,
        root: &Map<String, Value>,
        entities: &super::entity::EntityIndex,
    ) {
        for table in self.registry.tables() {
            let Some(rows) = value_at_path(root, table.payload_path()).and_then(Value::as_array)
            else {
                continue;
            };
            for (row_index, row) in rows.iter().enumerate() {
                let Some(row) = row.as_object() else { continue };
                for field in &table.row_fields {
                    let (Some(reference), Some(value)) =
                        (field.reference.as_ref(), row.get(&field.name))
                    else {
                        continue;
                    };
                    if !value.is_null()
                        && !reference_target_exists(root, self.registry, entities, reference, value)
                    {
                        self.error_with_context(
                            IFCCAD_IFCDR_REFERENCE_MISSING,
                            &format!(
                                "{}/{}/{}",
                                path_pointer(table.payload_path()),
                                row_index,
                                escape(&field.name)
                            ),
                            "table field references a missing target",
                            BTreeMap::from([(
                                "target".to_owned(),
                                PackageDiagnosticContextValue::String(reference.target.clone()),
                            )]),
                        );
                    }
                }
            }
        }

        let Some(payloads) = root.get("streams").and_then(Value::as_object) else {
            return;
        };
        for stream in self.registry.streams() {
            let Some(payload) = payloads
                .get(stream.payload_key())
                .and_then(Value::as_object)
            else {
                continue;
            };
            for column in &stream.columns {
                let Some(reference) = &column.reference else {
                    continue;
                };
                if reference.category == ReferenceCategory::Entity
                    && stream.role() == StreamRole::Object
                    && column.name == "entityId"
                {
                    continue;
                }
                let Some(values) = payload.get(&column.name).and_then(Value::as_array) else {
                    continue;
                };
                for (row, value) in values.iter().enumerate() {
                    if value.is_null() {
                        continue;
                    }
                    if !reference_target_exists(root, self.registry, entities, reference, value) {
                        self.error_with_context(
                            IFCCAD_IFCDR_REFERENCE_MISSING,
                            &format!(
                                "/streams/{}/{}/{}",
                                escape(stream.payload_key()),
                                escape(&column.name),
                                row
                            ),
                            "stream column references a missing target",
                            BTreeMap::from([(
                                "target".to_owned(),
                                PackageDiagnosticContextValue::String(reference.target.clone()),
                            )]),
                        );
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

fn reference_target_exists(
    root: &Map<String, Value>,
    registry: &IfcdrRegistry,
    entities: &super::entity::EntityIndex,
    reference: &super::registry::Reference,
    value: &Value,
) -> bool {
    match reference.category {
        ReferenceCategory::Ifcx => true,
        ReferenceCategory::Entity => value
            .as_u64()
            .and_then(crate::ifcdr::EntityId::new)
            .is_some_and(|id| entities.get(id).is_some()),
        ReferenceCategory::TableField => {
            reference
                .target
                .split_once('.')
                .is_some_and(|(payload_path, field)| {
                    registry.table_by_payload_path(payload_path).is_some()
                        && value_at_path(root, payload_path)
                            .and_then(Value::as_array)
                            .is_some_and(|rows| {
                                rows.iter().any(|row| row.get(field) == Some(value))
                            })
                })
        }
        ReferenceCategory::StreamColumn => {
            reference
                .target
                .split_once('.')
                .is_some_and(|(payload_key, column)| {
                    registry.stream_by_payload_key(payload_key).is_some()
                        && root
                            .get("streams")
                            .and_then(Value::as_object)
                            .and_then(|streams| streams.get(payload_key))
                            .and_then(Value::as_object)
                            .and_then(|payload| payload.get(column))
                            .and_then(Value::as_array)
                            .is_some_and(|values| values.contains(value))
                })
        }
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

#[cfg(test)]
pub(super) fn validate_value(uri: &str, value: Value) -> ValidationOutcome<LoadedIfcdrResource> {
    let source = super::resource::fixture_source();
    let source = Arc::new(source.with_test_value(value));
    validate_ifcdr(LoadedIfcdrResource::new(uri.to_owned(), source))
}

#[cfg(test)]
mod tests {
    use super::super::resource::fixture_source;
    use super::*;

    #[test]
    fn validates_the_minimal_ifcdr_resource() {
        let loaded = LoadedIfcdrResource::new("drawing.ifcdr.json".to_owned(), fixture_source());
        let outcome = validate_ifcdr(loaded);

        assert!(outcome.validated().is_some(), "{:?}", outcome.diagnostics());
        assert!(outcome.diagnostics().is_empty());
    }

    #[test]
    fn envelope_rejects_an_unsupported_version_without_cascading() {
        let mut value = fixture_source().value().clone();
        value["header"]["version"] = serde_json::json!("0.6.0");
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert_eq!(outcome.diagnostics().len(), 1);
        let diagnostic = &outcome.diagnostics()[0];
        assert_eq!(diagnostic.code, "IFCCAD_IFCDR_VERSION_UNSUPPORTED");
        assert_eq!(
            diagnostic.resource_uri.as_deref(),
            Some("drawing.ifcdr.json")
        );
        assert_eq!(diagnostic.location.as_deref(), Some("/header/version"));
    }

    #[test]
    fn table_rejects_a_duplicate_local_id() {
        let mut value = fixture_source().value().clone();
        value["layerBindings"][1]["id"] = serde_json::json!(0);
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_STRUCTURE_INVALID"
                && diagnostic.location.as_deref() == Some("/layerBindings/1/id")
        }));
    }

    #[test]
    fn directory_rejects_a_parent_that_disagrees_with_the_registry() {
        let mut value = fixture_source().value().clone();
        value["streamDirectory"]["streams"][3]["parent"] = serde_json::json!("line");
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_DIRECTORY_INVALID"
                && diagnostic.location.as_deref() == Some("/streamDirectory/streams/3/parent")
        }));
    }

    #[test]
    fn directory_rejects_a_non_string_column_name() {
        let mut value = fixture_source().value().clone();
        value["streamDirectory"]["streams"][0]["columns"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!(7));
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_DIRECTORY_INVALID"
                && diagnostic.location.as_deref() == Some("/streamDirectory/streams/0/columns/8")
        }));
    }

    #[test]
    fn directory_rejects_a_duplicate_column_name() {
        let mut value = fixture_source().value().clone();
        value["streamDirectory"]["streams"][0]["columns"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!("entityId"));
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_DIRECTORY_INVALID"
                && diagnostic.location.as_deref() == Some("/streamDirectory/streams/0/columns/8")
        }));
    }

    #[test]
    fn directory_rejects_a_non_string_child_name() {
        let mut value = fixture_source().value().clone();
        value["streamDirectory"]["streams"][2]["children"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!(7));
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_DIRECTORY_INVALID"
                && diagnostic.location.as_deref() == Some("/streamDirectory/streams/2/children/1")
        }));
    }

    #[test]
    fn stream_rejects_a_missing_table_reference() {
        let mut value = fixture_source().value().clone();
        value["streams"]["lineStream"]["layerId"][0] = serde_json::json!(99);
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_REFERENCE_MISSING"
                && diagnostic.location.as_deref() == Some("/streams/lineStream/layerId/0")
        }));
    }

    #[test]
    fn stream_rejects_a_pool_range_past_the_column_end() {
        let mut value = fixture_source().value().clone();
        value["streams"]["polylineStream"]["vertexCount"][1] = serde_json::json!(4);
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_STRUCTURE_INVALID"
                && diagnostic.location.as_deref() == Some("/streams/polylineStream/vertexCount/1")
        }));
    }

    #[test]
    fn stream_rejects_an_external_range_past_its_target_payload() {
        let mut value = fixture_source().value().clone();
        value["streams"]["entityOrderStream"]["entryCount"][0] = serde_json::json!(5);
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_STRUCTURE_INVALID"
                && diagnostic.location.as_deref() == Some("/streams/entityOrderStream/entryCount/0")
        }));
    }

    #[test]
    fn stream_rejects_a_nonempty_range_with_a_missing_external_target() {
        let mut value = fixture_source().value().clone();
        value["streamDirectory"]["streams"]
            .as_array_mut()
            .unwrap()
            .retain(|entry| entry["name"] != "entityOrderEntry");
        value["streamDirectory"]["streams"][2]["children"] = serde_json::json!([]);
        value["streams"]
            .as_object_mut()
            .unwrap()
            .remove("entityOrderEntryStream");
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_STRUCTURE_INVALID"
                && diagnostic.location.as_deref() == Some("/streams/entityOrderStream/entryCount/0")
        }));
    }

    #[test]
    fn table_rejects_a_missing_reference_target() {
        let mut value = fixture_source().value().clone();
        value["appearanceBindings"][0]["overrideId"] = serde_json::json!(99);
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_REFERENCE_MISSING"
                && diagnostic.location.as_deref() == Some("/appearanceBindings/0/overrideId")
        }));
    }

    #[test]
    fn stream_rejects_unsynchronized_pool_column_lengths() {
        let mut value = fixture_source().value().clone();
        value["streams"]["polylineStream"]["y"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!(40.0));
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_STRUCTURE_INVALID"
                && diagnostic.location.as_deref() == Some("/streams/polylineStream/y")
        }));
    }
}
